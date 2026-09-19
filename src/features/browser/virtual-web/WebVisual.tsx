// Shared code-native illustration library. No remote assets or executable pack markup.
export function WebVisual({ kind, large = false }: { kind: string; large?: boolean }) {
  return (
    <div
      className={`web-visual web-visual-${kind}${large ? ' web-visual-large' : ''}`}
      aria-hidden="true"
    >
      <svg viewBox="0 0 320 210" fill="none">
        <ellipse cx="160" cy="176" rx="97" ry="12" fill="currentColor" opacity=".09" />
        {kind === 'duck' ? (
          <>
            <path
              d="M91 126c-19-9-32-9-44-6 12 37 51 54 107 48 43-5 63-24 64-48 1-18-13-28-29-28-8 0-15 3-22 9-15 17-47 28-76 25Z"
              fill="#f5c543"
            />
            <circle cx="195" cy="80" r="36" fill="#ffd753" />
            <path d="m224 84 34 10-36 13Z" fill="#e98736" />
            <circle cx="204" cy="72" r="4" fill="#3e3829" />
            <path
              d="M113 141c24 8 48 4 64-11"
              stroke="#da9f30"
              strokeWidth="5"
              strokeLinecap="round"
            />
          </>
        ) : kind === 'phone' ? (
          <>
            <rect x="119" y="23" width="83" height="156" rx="15" fill="#303844" />
            <rect x="125" y="30" width="71" height="140" rx="10" fill="#9badcd" />
            <path d="M125 116c24-40 48-22 71-70v114c-24 20-50 3-71 10Z" fill="#dce6f0" />
            <rect x="144" y="34" width="34" height="6" rx="3" fill="#303844" />
            <circle cx="163" cy="153" r="4" fill="#8592a9" />
          </>
        ) : kind === 'router' ? (
          <>
            <path
              d="M105 132 91 52m123 80 17-80"
              stroke="#424f60"
              strokeWidth="8"
              strokeLinecap="round"
            />
            <path d="m75 140 34-25h115l23 28-8 28H82Z" fill="#e3e9ed" />
            <path d="M82 151h157v20H82Z" fill="#becbd4" />
            <circle cx="110" cy="157" r="3" fill="#28958a" />
            <circle cx="122" cy="157" r="3" fill="#28958a" />
            <path
              d="M130 69q32-26 64 0m-53 14q21-16 42 0m-28 14q8-7 16 0"
              stroke="#83a9bc"
              strokeWidth="5"
              strokeLinecap="round"
            />
          </>
        ) : kind === 'cable' ? (
          <>
            <path
              d="M83 61c-60 150 175 144 147 30-9-33-83-47-91 0-8 52 58 67 57 27"
              stroke="#5c6a86"
              strokeWidth="12"
              strokeLinecap="round"
            />
            <rect x="68" y="41" width="28" height="36" rx="7" fill="#343f57" />
            <rect x="73" y="27" width="18" height="20" rx="5" fill="#b8c5cc" />
            <rect x="182" y="96" width="28" height="34" rx="7" fill="#343f57" />
            <rect x="187" y="83" width="18" height="20" rx="5" fill="#b8c5cc" />
          </>
        ) : kind === 'chair' ? (
          <>
            <rect x="116" y="24" width="88" height="95" rx="23" fill="#496074" />
            <rect x="129" y="36" width="62" height="60" rx="14" fill="#627e8e" />
            <path
              d="M102 95v34m116-34v34m-112 0h108"
              stroke="#374958"
              strokeWidth="10"
              strokeLinecap="round"
            />
            <rect x="107" y="116" width="108" height="23" rx="11" fill="#708b99" />
            <path
              d="M160 139v35m-45 6 45-10 45 10"
              stroke="#87969d"
              strokeWidth="8"
              strokeLinecap="round"
            />
            <circle cx="112" cy="182" r="7" fill="#374958" />
            <circle cx="208" cy="182" r="7" fill="#374958" />
          </>
        ) : kind === 'car' ? (
          <>
            <path d="m57 113 29-10 35-43h81l34 43 26 12v42H57Z" fill="#719ba2" />
            <path d="m101 100 28-31h29v31Zm66-31h29l27 31h-56Z" fill="#dae8e7" />
            <path d="M59 146h203" stroke="#4f747e" strokeWidth="9" />
            <rect x="59" y="119" width="24" height="11" rx="4" fill="#f5dc97" />
            <rect x="241" y="119" width="17" height="11" rx="3" fill="#b96754" />
            <circle cx="104" cy="154" r="23" fill="#344954" />
            <circle cx="217" cy="154" r="23" fill="#344954" />
            <circle cx="104" cy="154" r="11" fill="#bdcbd0" />
            <circle cx="217" cy="154" r="11" fill="#bdcbd0" />
          </>
        ) : kind === 'paw' ? (
          <>
            <path d="M113 116q-20 39-7 59h106q10-40-14-61Z" fill="#bd946b" />
            <path
              d="M119 59q-37-12-38 20t28 53l20-56m62-17q37-12 38 20t-28 53l-20-56"
              fill="#775843"
            />
            <path d="M111 80q0-39 47-39t47 39v29q-7 42-47 42t-47-42Z" fill="#d7b78d" />
            <ellipse cx="157" cy="116" rx="28" ry="22" fill="#f1dfbf" />
            <circle cx="134" cy="87" r="5" fill="#423e38" />
            <circle cx="179" cy="87" r="5" fill="#423e38" />
            <path d="M149 108q9-8 18 0l-9 10Z" fill="#423e38" />
            <path
              d="M158 118v10m-12-3q12 11 24 0"
              stroke="#775843"
              strokeWidth="3"
              strokeLinecap="round"
            />
            <path d="M124 148q32 17 65 0" stroke="#5e8c94" strokeWidth="9" />
          </>
        ) : kind === 'radio' ? (
          <>
            <path d="m214 54 36-31" stroke="#95a9af" strokeWidth="5" strokeLinecap="round" />
            <path d="M121 60V44h80v16" stroke="#6d5744" strokeWidth="9" strokeLinecap="round" />
            <rect x="60" y="61" width="205" height="108" rx="15" fill="#b8875c" />
            <circle cx="117" cy="116" r="38" fill="#654f41" />
            <circle cx="117" cy="116" r="28" stroke="#9c7c5f" strokeWidth="4" />
            <rect x="170" y="79" width="75" height="28" rx="4" fill="#ead6ad" />
            <path d="M180 92h53m-32-9v18" stroke="#886647" strokeWidth="3" />
            <circle cx="188" cy="137" r="14" fill="#ead6ad" />
            <circle cx="230" cy="137" r="10" fill="#ead6ad" />
          </>
        ) : kind === 'cake' || kind === 'pizza' || kind === 'bowl' || kind === 'cup' ? (
          <>
            <ellipse cx="162" cy="151" rx="101" ry="28" fill="#fff9ec" />
            <ellipse cx="162" cy="145" rx="89" ry="24" stroke="#d8c3a6" strokeWidth="2" />
            {kind === 'cake' ? (
              <>
                <path d="m103 136 8-58h100l10 58q-58 30-118 0" fill="#dfa35b" />
                <ellipse cx="161" cy="79" rx="50" ry="18" fill="#614432" />
                <path
                  d="M114 86v29m25-21v11m57-17v22"
                  stroke="#614432"
                  strokeWidth="9"
                  strokeLinecap="round"
                />
                <ellipse cx="161" cy="80" rx="15" ry="7" fill="#d9b887" />
              </>
            ) : kind === 'pizza' ? (
              <>
                <ellipse cx="162" cy="121" rx="83" ry="38" fill="#d8a561" />
                <ellipse cx="162" cy="117" rx="74" ry="32" fill="#e8c17a" />
                {[112, 150, 193, 175, 132, 210].map((x, i) => (
                  <ellipse key={x} cx={x} cy={103 + (i % 3) * 13} rx="10" ry="6" fill="#b9563b" />
                ))}
              </>
            ) : kind === 'bowl' ? (
              <>
                <path d="M82 99h160q-7 64-80 64T82 99" fill="#83a59e" />
                <ellipse cx="162" cy="99" rx="80" ry="25" fill="#e7d7a7" />
                <ellipse cx="145" cy="95" rx="24" ry="9" fill="#779157" />
                <ellipse cx="186" cy="106" rx="23" ry="8" fill="#ca7950" />
                <path d="m197 80 37-36" stroke="#a5977d" strokeWidth="7" strokeLinecap="round" />
              </>
            ) : (
              <>
                <path d="M111 77h96l-10 67q-39 24-77 0Z" fill="#f3ede0" />
                <ellipse cx="159" cy="78" rx="48" ry="13" fill="#715744" />
                <path d="M207 89c41-5 34 49-6 35" stroke="#e0d7c7" strokeWidth="10" />
                <path
                  d="M149 58c-12-16 15-13 3-28m22 28c-12-16 15-13 3-28"
                  stroke="#baa28a"
                  strokeWidth="3"
                  strokeLinecap="round"
                />
              </>
            )}
          </>
        ) : kind === 'chip' ? (
          <>
            <rect x="59" y="53" width="206" height="114" rx="8" fill="#416f65" />
            <path d="M77 167v10h143v-10" fill="#c6a45c" />
            <rect x="76" y="70" width="140" height="79" rx="8" fill="#354c50" />
            <circle cx="146" cy="109" r="32" fill="#90a9a7" />
            <path
              d="M146 82v54m-27-27h54m-46-19 38 38m0-38-38 38"
              stroke="#3b5559"
              strokeWidth="8"
            />
            <circle cx="146" cy="109" r="10" fill="#b4c9c4" />
            <rect x="230" y="75" width="20" height="45" rx="3" fill="#a0b6ac" />
            <path d="M54 46v127" stroke="#aab8b9" strokeWidth="6" />
          </>
        ) : kind === 'code' ? (
          <>
            <rect x="59" y="46" width="204" height="124" rx="10" fill="#34475b" />
            <rect x="66" y="54" width="190" height="105" rx="5" fill="#233546" />
            <circle cx="78" cy="64" r="3" fill="#d38369" />
            <circle cx="89" cy="64" r="3" fill="#eac779" />
            <circle cx="100" cy="64" r="3" fill="#81ad95" />
            <path
              d="m130 91-20 20 20 20m64-40 20 20-20 20m-26-48-15 56"
              stroke="#9ad2c7"
              strokeWidth="7"
              strokeLinecap="round"
            />
            <path d="M41 170h241l-19 9H60Z" fill="#b0c2cc" />
          </>
        ) : kind === 'film' ? (
          <>
            <rect x="58" y="49" width="207" height="127" rx="6" fill="#2e394d" />
            <path d="m74 66 167 92M65 98l145 78" stroke="#576181" strokeWidth="2" />
            <circle cx="166" cy="99" r="31" fill="#a7b6c6" opacity=".7" />
            <path d="M130 176c2-40 18-57 35-57s33 17 36 57" fill="#1c293b" />
            <path
              d="m78 41 43-21 16 30 42-21 16 30 42-21 15 27"
              stroke="#a0adb9"
              strokeWidth="11"
            />
          </>
        ) : kind === 'office' || kind === 'house' ? (
          <>
            <path d="M63 99 126 51l47 38 31-28 55 37v73H63Z" fill="#90a7b6" />
            <path d="M82 108h41v63H82Zm65 0h31v63h-31Zm56 0h36v63h-36Z" fill="#dfe6e7" />
            <path d="M60 174h204" stroke="#5f7c83" strokeWidth="5" />
            <circle cx="53" cy="114" r="25" fill="#719c8c" />
            <path d="M53 131v44" stroke="#55706b" strokeWidth="6" />
          </>
        ) : (
          <>
            <path
              d="M64 65c34-10 66-6 96 10 29-16 61-20 95-10v99c-34-8-66-4-95 11-30-15-62-19-96-11Z"
              fill="#f3efe3"
            />
            <path d="M160 75v100" stroke="#c7b9a5" strokeWidth="3" />
            <path
              d="M80 87q36-6 65 8m-65 14q36-6 65 8m-65 14q36-6 65 8m32-44q29-14 62-8m-62 30q29-14 62-8m-62 30q29-14 62-8"
              stroke="#a8b8b8"
              strokeWidth="4"
            />
            <path d="M64 164v7q53-8 96 12 43-20 95-12v-7" stroke="#74928c" strokeWidth="5" />
          </>
        )}
      </svg>
    </div>
  );
}
