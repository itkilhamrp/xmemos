// Copyright (C) 2026 Xmemos Contributors
//
// This file is part of Xmemos.
//
// Xmemos is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Xmemos is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Xmemos. If not, see https://www.gnu.org/licenses/.
//
// SPDX-License-Identifier: AGPL-3.0-or-later

package org.xmemos.storage

import android.content.Context
import android.media.MediaMetadataRetriever
import android.net.Uri
import android.os.Handler
import android.os.Looper
import androidx.annotation.OptIn
import androidx.media3.common.Effect
import androidx.media3.common.MediaItem
import androidx.media3.common.MimeTypes
import androidx.media3.common.util.UnstableApi
import androidx.media3.effect.Presentation
import androidx.media3.transformer.Composition
import androidx.media3.transformer.DefaultEncoderFactory
import androidx.media3.transformer.EditedMediaItem
import androidx.media3.transformer.Effects
import androidx.media3.transformer.ExportException
import androidx.media3.transformer.ExportResult
import androidx.media3.transformer.Transformer
import androidx.media3.transformer.VideoEncoderSettings
import java.io.File
import java.util.UUID
import java.util.concurrent.atomic.AtomicBoolean

internal data class VideoCompressionOutcome(
    val compressed: Boolean,
    val outputPath: String,
    val mimeType: String,
    val originalBytes: Long,
    val outputBytes: Long,
)

/**
 * Hardware-accelerated, fail-safe Android video compressor powered by Jetpack Media3 Transformer
 * (Apache 2.0 FOSS, zero GMS). Uses device MediaCodec hardware encoders, preserves portrait/landscape
 * orientation via Presentation.createForShortSide, enforces a 90-second watchdog timeout, and
 * guarantees zero bloat or data loss by falling back to the original file whenever compression fails
 * or does not save at least 8% of file size.
 */
@OptIn(UnstableApi::class)
internal object XmemosVideoCompressor {
    private const val MIN_COMPRESS_BYTES = 1_500_000L // Skip tiny clips (<1.5 MB)
    private const val DEFAULT_MAX_SHORT_SIDE = 720
    private const val DEFAULT_MAX_BITRATE_BPS = 2_200_000
    private const val MIN_BITRATE_BPS = 900_000
    private const val ALREADY_COMPACT_BITRATE_BPS = 1_600_000
    private const val MAX_DURATION_MS = 15L * 60L * 1000L // Skip >15 min movies to avoid long waits
    private const val TIMEOUT_MS = 90_000L

    private data class VideoProbe(
        val shortSide: Int,
        val bitrateBps: Int,
        val durationMs: Long,
    )

    private fun probeVideo(file: File): VideoProbe {
        val retriever = MediaMetadataRetriever()
        return try {
            retriever.setDataSource(file.absolutePath)
            val width = retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_VIDEO_WIDTH)
                ?.toIntOrNull() ?: 0
            val height = retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_VIDEO_HEIGHT)
                ?.toIntOrNull() ?: 0
            val bitrate = retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_BITRATE)
                ?.toIntOrNull() ?: 0
            val duration = retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_DURATION)
                ?.toLongOrNull() ?: 0L
            val shortSide = if (width > 0 && height > 0) minOf(width, height) else 0
            VideoProbe(shortSide = shortSide, bitrateBps = bitrate, durationMs = duration)
        } catch (_: Throwable) {
            VideoProbe(shortSide = 0, bitrateBps = 0, durationMs = 0L)
        } finally {
            runCatching { retriever.release() }
        }
    }

    fun compress(
        context: Context,
        sourcePath: String,
        maxShortSideArg: Int?,
        targetBitrateArg: Int?,
        onDone: (VideoCompressionOutcome) -> Unit,
    ) {
        val inputFile = File(sourcePath)
        val origBytes = if (inputFile.exists() && inputFile.isFile) inputFile.length() else 0L
        fun passthrough(): VideoCompressionOutcome = VideoCompressionOutcome(
            compressed = false,
            outputPath = sourcePath,
            mimeType = "video/mp4",
            originalBytes = origBytes,
            outputBytes = origBytes,
        )

        if (!inputFile.exists() || !inputFile.isFile || origBytes < MIN_COMPRESS_BYTES) {
            onDone(passthrough())
            return
        }

        val maxShortSide = (maxShortSideArg ?: DEFAULT_MAX_SHORT_SIDE).coerceIn(480, 1080)
        val maxBitrate = (targetBitrateArg ?: DEFAULT_MAX_BITRATE_BPS).coerceIn(MIN_BITRATE_BPS, 4_000_000)

        val probe = probeVideo(inputFile)
        if (probe.durationMs > MAX_DURATION_MS) {
            onDone(passthrough())
            return
        }
        if (probe.shortSide in 1..maxShortSide && probe.bitrateBps in 1..ALREADY_COMPACT_BITRATE_BPS) {
            onDone(passthrough())
            return
        }

        val adaptiveBitrate = if (probe.bitrateBps > 0) {
            (probe.bitrateBps * 0.55).toInt().coerceIn(MIN_BITRATE_BPS, maxBitrate)
        } else {
            maxBitrate
        }

        val stageDir = File(context.cacheDir, "video_opt").apply { mkdirs() }
        val outputFile = File(stageDir, "opt_${UUID.randomUUID()}.mp4")
        val appContext = context.applicationContext
        val mainHandler = Handler(Looper.getMainLooper())
        val finished = AtomicBoolean(false)

        mainHandler.post {
            try {
                val videoEffects = mutableListOf<Effect>()
                if (probe.shortSide > maxShortSide) {
                    videoEffects.add(Presentation.createForShortSide(maxShortSide))
                }

                val editedItem = EditedMediaItem.Builder(MediaItem.fromUri(Uri.fromFile(inputFile)))
                    .setEffects(Effects(emptyList(), videoEffects))
                    .build()

                val encoderFactory = DefaultEncoderFactory.Builder(appContext)
                    .setEnableFallback(true)
                    .setRequestedVideoEncoderSettings(
                        VideoEncoderSettings.Builder()
                            .setBitrate(adaptiveBitrate)
                            .build()
                    )
                    .build()

                var transformerRef: Transformer? = null
                val timeoutRunnable = Runnable {
                    if (finished.compareAndSet(false, true)) {
                        runCatching { transformerRef?.cancel() }
                        runCatching { outputFile.delete() }
                        onDone(passthrough())
                    }
                }

                val transformer = Transformer.Builder(appContext)
                    .setVideoMimeType(MimeTypes.VIDEO_H264)
                    .setAudioMimeType(MimeTypes.AUDIO_AAC)
                    .setEncoderFactory(encoderFactory)
                    .addListener(object : Transformer.Listener {
                        override fun onCompleted(composition: Composition, exportResult: ExportResult) {
                            mainHandler.removeCallbacks(timeoutRunnable)
                            if (!finished.compareAndSet(false, true)) return
                            val outBytes = if (outputFile.exists() && outputFile.isFile) outputFile.length() else 0L
                            // Anti-bloat & integrity guard: require at least 8% size reduction
                            if (outBytes > 0L && outBytes < (origBytes * 92L / 100L)) {
                                onDone(
                                    VideoCompressionOutcome(
                                        compressed = true,
                                        outputPath = outputFile.absolutePath,
                                        mimeType = MimeTypes.VIDEO_MP4,
                                        originalBytes = origBytes,
                                        outputBytes = outBytes,
                                    )
                                )
                            } else {
                                runCatching { outputFile.delete() }
                                onDone(passthrough())
                            }
                        }

                        override fun onError(
                            composition: Composition,
                            exportResult: ExportResult,
                            exportException: ExportException,
                        ) {
                            mainHandler.removeCallbacks(timeoutRunnable)
                            if (!finished.compareAndSet(false, true)) return
                            runCatching { outputFile.delete() }
                            onDone(passthrough())
                        }
                    })
                    .build()

                transformerRef = transformer
                mainHandler.postDelayed(timeoutRunnable, TIMEOUT_MS)
                transformer.start(editedItem, outputFile.absolutePath)
            } catch (_: Throwable) {
                if (finished.compareAndSet(false, true)) {
                    runCatching { outputFile.delete() }
                    onDone(passthrough())
                }
            }
        }
    }
}
