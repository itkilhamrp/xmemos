<!--
  Copyright (C) 2026 Xmemos Contributors

  This file is part of Xmemos.

  Xmemos is free software: you can redistribute it and/or modify
  it under the terms of the GNU Affero General Public License as published by
  the Free Software Foundation, either version 3 of the License, or
  (at your option) any later version.

  Xmemos is distributed in the hope that it will be useful,
  but WITHOUT ANY WARRANTY; without even the implied warranty of
  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
  GNU Affero General Public License for more details.

  You should have received a copy of the GNU Affero General Public License
  along with Xmemos. If not, see https://www.gnu.org/licenses/.

  SPDX-License-Identifier: AGPL-3.0-or-later
-->

<script lang="ts">
	interface Props {
		class?: string;
		ariaLabel?: string;
		decorative?: boolean;
		dragRegion?: boolean;
	}

	let {
		class: className = 'h-14 w-14',
		ariaLabel = 'Xmemos',
		decorative = false,
		dragRegion = false
	}: Props = $props();

	const uid = $props.id();
	const squircleOuter =
		'M 512.00 80.00 C 754.78 80.00, 822.23 79.08, 883.57 140.43 C 944.92 201.77, 944.00 269.22, 944.00 512.00 C 944.00 754.78, 944.92 822.23, 883.57 883.57 C 822.23 944.92, 754.78 944.00, 512.00 944.00 C 269.22 944.00, 201.77 944.92, 140.43 883.57 C 79.08 822.23, 80.00 754.78, 80.00 512.00 C 80.00 269.22, 79.08 201.77, 140.43 140.43 C 201.77 79.08, 269.22 80.00, 512.00 80.00 Z';
	const squircleInner =
		'M 512.00 86.00 C 751.41 86.00, 817.92 85.10, 878.41 145.59 C 938.90 206.08, 938.00 272.59, 938.00 512.00 C 938.00 751.41, 938.90 817.92, 878.41 878.41 C 817.92 938.90, 751.41 938.00, 512.00 938.00 C 272.59 938.00, 206.08 938.90, 145.59 878.41 C 85.10 817.92, 86.00 751.41, 86.00 512.00 C 86.00 272.59, 85.10 206.08, 145.59 145.59 C 206.08 85.10, 272.59 86.00, 512.00 86.00 Z';
</script>

<svg
	xmlns="http://www.w3.org/2000/svg"
	viewBox="0 0 1024 1024"
	fill="none"
	class={className}
	role={decorative ? undefined : 'img'}
	aria-label={decorative ? undefined : ariaLabel}
	aria-hidden={decorative ? 'true' : undefined}
	data-tauri-drag-region={dragRegion ? true : undefined}
>
	<defs>
		<linearGradient id="{uid}-bg-base" x1="110" y1="80" x2="914" y2="944" gradientUnits="userSpaceOnUse">
			<stop offset="0%" stop-color="#3AC0FF" />
			<stop offset="30%" stop-color="#0A84FF" />
			<stop offset="70%" stop-color="#0056D2" />
			<stop offset="100%" stop-color="#06287A" />
		</linearGradient>

		<radialGradient id="{uid}-caustic-top" cx="270" cy="185" r="590" gradientUnits="userSpaceOnUse">
			<stop offset="0%" stop-color="#A5F3FC" stop-opacity="0.62" />
			<stop offset="40%" stop-color="#38BDF8" stop-opacity="0.24" />
			<stop offset="100%" stop-color="#0A84FF" stop-opacity="0" />
		</radialGradient>

		<radialGradient id="{uid}-caustic-bottom" cx="775" cy="865" r="480" gradientUnits="userSpaceOnUse">
			<stop offset="0%" stop-color="#60A5FA" stop-opacity="0.56" />
			<stop offset="50%" stop-color="#2563EB" stop-opacity="0.18" />
			<stop offset="100%" stop-color="#06287A" stop-opacity="0" />
		</radialGradient>

		<radialGradient
			id="{uid}-glass-sheen"
			cx="512"
			cy="110"
			r="620"
			fx="440"
			fy="90"
			gradientUnits="userSpaceOnUse"
		>
			<stop offset="0%" stop-color="#FFFFFF" stop-opacity="0.46" />
			<stop offset="42%" stop-color="#E0F2FE" stop-opacity="0.14" />
			<stop offset="76%" stop-color="#FFFFFF" stop-opacity="0" />
			<stop offset="100%" stop-color="#FFFFFF" stop-opacity="0" />
		</radialGradient>

		<linearGradient id="{uid}-bottom-bounce" x1="512" y1="730" x2="512" y2="944" gradientUnits="userSpaceOnUse">
			<stop offset="0%" stop-color="#BAE6FD" stop-opacity="0" />
			<stop offset="65%" stop-color="#BAE6FD" stop-opacity="0.14" />
			<stop offset="100%" stop-color="#E0F2FE" stop-opacity="0.34" />
		</linearGradient>

		<linearGradient id="{uid}-rim-bevel" x1="120" y1="90" x2="904" y2="934" gradientUnits="userSpaceOnUse">
			<stop offset="0%" stop-color="#FFFFFF" stop-opacity="0.92" />
			<stop offset="26%" stop-color="#E0F2FE" stop-opacity="0.40" />
			<stop offset="58%" stop-color="#FFFFFF" stop-opacity="0.12" />
			<stop offset="100%" stop-color="#93C5FD" stop-opacity="0.68" />
		</linearGradient>

		<linearGradient id="{uid}-back-sheet" x1="292" y1="202" x2="732" y2="330" gradientUnits="userSpaceOnUse">
			<stop offset="0%" stop-color="#FFFFFF" stop-opacity="0.52" />
			<stop offset="100%" stop-color="#BAE6FD" stop-opacity="0.20" />
		</linearGradient>

		<linearGradient id="{uid}-card-fill" x1="256" y1="238" x2="768" y2="790" gradientUnits="userSpaceOnUse">
			<stop offset="0%" stop-color="#FFFFFF" stop-opacity="0.99" />
			<stop offset="50%" stop-color="#F3F9FF" stop-opacity="0.96" />
			<stop offset="100%" stop-color="#DCEBFF" stop-opacity="0.92" />
		</linearGradient>

		<linearGradient id="{uid}-card-rim" x1="256" y1="238" x2="768" y2="790" gradientUnits="userSpaceOnUse">
			<stop offset="0%" stop-color="#FFFFFF" stop-opacity="1" />
			<stop offset="52%" stop-color="#E0F2FE" stop-opacity="0.58" />
			<stop offset="100%" stop-color="#93C5FD" stop-opacity="0.78" />
		</linearGradient>

		<linearGradient id="{uid}-fold-flap" x1="630" y1="238" x2="768" y2="376" gradientUnits="userSpaceOnUse">
			<stop offset="0%" stop-color="#E0F2FE" />
			<stop offset="48%" stop-color="#93C5FD" />
			<stop offset="100%" stop-color="#60A5FA" />
		</linearGradient>

		<linearGradient id="{uid}-x-back" x1="634" y1="422" x2="390" y2="666" gradientUnits="userSpaceOnUse">
			<stop offset="0%" stop-color="#0EA5E9" />
			<stop offset="48%" stop-color="#005ED9" />
			<stop offset="100%" stop-color="#08308A" />
		</linearGradient>

		<linearGradient id="{uid}-x-front" x1="390" y1="422" x2="634" y2="666" gradientUnits="userSpaceOnUse">
			<stop offset="0%" stop-color="#38BDF8" />
			<stop offset="44%" stop-color="#0A84FF" />
			<stop offset="100%" stop-color="#004EC2" />
		</linearGradient>

		<linearGradient id="{uid}-x-sheen" x1="396" y1="414" x2="628" y2="646" gradientUnits="userSpaceOnUse">
			<stop offset="0%" stop-color="#FFFFFF" stop-opacity="0.82" />
			<stop offset="52%" stop-color="#E0F2FE" stop-opacity="0.20" />
			<stop offset="100%" stop-color="#FFFFFF" stop-opacity="0" />
		</linearGradient>

		<filter id="{uid}-tile-shadow" filterUnits="userSpaceOnUse" x="0" y="0" width="1024" height="1024">
			<feDropShadow dx="0" dy="18" stdDeviation="20" flood-color="#00163E" flood-opacity="0.32" />
		</filter>

		<filter id="{uid}-card-shadow" filterUnits="userSpaceOnUse" x="140" y="140" width="744" height="780">
			<feDropShadow dx="0" dy="20" stdDeviation="20" flood-color="#001C54" flood-opacity="0.30" />
		</filter>

		<filter id="{uid}-fold-shadow" filterUnits="userSpaceOnUse" x="580" y="200" width="240" height="240">
			<feDropShadow dx="-4" dy="8" stdDeviation="7" flood-color="#002D7A" flood-opacity="0.22" />
		</filter>

		<filter id="{uid}-x-cross-shadow" filterUnits="userSpaceOnUse" x="300" y="340" width="424" height="424">
			<feDropShadow dx="0" dy="8" stdDeviation="8" flood-color="#001B4F" flood-opacity="0.28" />
		</filter>

		<clipPath id="{uid}-squircle-clip">
			<path d={squircleOuter} />
		</clipPath>
	</defs>

	<g filter="url(#{uid}-tile-shadow)">
		<path d={squircleOuter} fill="url(#{uid}-bg-base)" />
	</g>

	<g clip-path="url(#{uid}-squircle-clip)">
		<rect x="80" y="80" width="864" height="864" fill="url(#{uid}-caustic-top)" />
		<rect x="80" y="80" width="864" height="864" fill="url(#{uid}-caustic-bottom)" />
		<rect x="80" y="80" width="864" height="864" fill="url(#{uid}-glass-sheen)" />
		<rect x="80" y="720" width="864" height="224" fill="url(#{uid}-bottom-bounce)" />
		<path d={squircleInner} stroke="url(#{uid}-rim-bevel)" stroke-width="6.5" />
		<path d={squircleOuter} stroke="#001842" stroke-opacity="0.26" stroke-width="4" />
	</g>

	<rect
		x="296"
		y="198"
		width="432"
		height="128"
		rx="52"
		fill="url(#{uid}-back-sheet)"
		stroke="#FFFFFF"
		stroke-opacity="0.50"
		stroke-width="2.5"
	/>

	<g filter="url(#{uid}-card-shadow)">
		<path
			d="M 332 238 H 630 L 768 376 V 714 C 768 756 734 790 692 790 H 332 C 290 790 256 756 256 714 V 314 C 256 272 290 238 332 238 Z"
			fill="url(#{uid}-card-fill)"
		/>
		<path
			d="M 332 240 H 629 L 766 377 V 714 C 766 754.9 732.9 788 692 788 H 332 C 291.1 788 258 754.9 258 714 V 314 C 258 273.1 291.1 240 332 240 Z"
			stroke="url(#{uid}-card-rim)"
			stroke-width="4"
		/>
	</g>

	<g filter="url(#{uid}-fold-shadow)">
		<path d="M 630 238 V 332 C 630 356.3 649.7 376 674 376 H 768 Z" fill="url(#{uid}-fold-flap)" />
		<path
			d="M 632 242.8 V 332 C 632 355.2 650.8 374 674 374 H 763.2 Z"
			stroke="#FFFFFF"
			stroke-opacity="0.76"
			stroke-width="2.5"
			stroke-linejoin="round"
		/>
	</g>

	<g>
		<path d="M 630 426 L 394 662" stroke="url(#{uid}-x-back)" stroke-width="86" stroke-linecap="round" />
		<g filter="url(#{uid}-x-cross-shadow)">
			<path d="M 394 426 L 630 662" stroke="url(#{uid}-x-front)" stroke-width="86" stroke-linecap="round" />
		</g>
		<path d="M 402 416 L 626 640" stroke="url(#{uid}-x-sheen)" stroke-width="11" stroke-linecap="round" />
	</g>
</svg>
