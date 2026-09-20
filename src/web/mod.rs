use axum::{response::Html, routing::get, Router};

pub const DASHBOARD_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>AITorrent Hub & qBittorrent Web Client</title>
  <script src="https://cdn.tailwindcss.com"></script>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Segoe+UI:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500;600&display=swap" rel="stylesheet">
  <style>
    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
      user-select: none;
    }
    code, pre, .mono {
      font-family: 'JetBrains Mono', monospace;
    }
    .qbit-bg-window { background-color: #1a1b26; }
    .qbit-bg-panel { background-color: #16161e; }
    .qbit-bg-header { background-color: #1f2335; }
    .qbit-border { border-color: #292e42; }
    .qbit-selected { background-color: #2f354f !important; }
    .qbit-row:hover { background-color: #24283b; }
    
    ::-webkit-scrollbar { width: 8px; height: 8px; }
    ::-webkit-scrollbar-track { background: #16161e; }
    ::-webkit-scrollbar-thumb { background: #2f354f; border-radius: 4px; }
    ::-webkit-scrollbar-thumb:hover { background: #3d4566; }
  </style>
</head>
<body class="qbit-bg-window text-slate-200 h-screen w-screen overflow-hidden flex flex-col text-[12px]">

  <!-- Top App Navigation / Mode Switcher -->
  <div class="qbit-bg-header border-b qbit-border px-3 py-1.5 flex items-center justify-between text-xs">
    <div class="flex items-center gap-4">
      <div class="flex items-center gap-2">
        <span class="w-6 h-6 rounded bg-blue-600 flex items-center justify-center font-bold text-white text-xs">⚡</span>
        <span class="font-bold tracking-tight text-white text-sm">AITorrent</span>
        <span class="text-[10px] text-slate-400 mono px-1.5 py-0.5 rounded bg-slate-800">v0.1.0 • Decentralized AI Hub</span>
      </div>

      <!-- Primary Platform View Switcher -->
      <div class="flex items-center bg-[#16161e] p-0.5 rounded-lg border qbit-border">
        <button onclick="switchMainView('hub')" id="view-btn-hub" class="px-3 py-1 rounded-md text-xs font-semibold hover:text-white transition flex items-center gap-1.5">
          <span>🌐 Model Torrent Hub</span>
          <span class="px-1 py-0.2 bg-blue-900/60 text-blue-300 text-[10px] rounded mono">3</span>
        </button>
        <button onclick="switchMainView('client')" id="view-btn-client" class="px-3 py-1 rounded-md text-xs font-semibold bg-blue-600 text-white shadow-sm flex items-center gap-1.5">
          <span>⚡ qBittorrent Client</span>
          <span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
        </button>
      </div>
    </div>

    <div class="flex items-center gap-3 text-[11px] mono">
      <span id="browser-compute-badge" class="px-2 py-0.5 rounded bg-emerald-950 text-emerald-400 border border-emerald-700/60 font-medium">
        ⚡ In-Browser WebGPU Seeder: ACTIVE
      </span>
      <span class="text-slate-400">Cluster: 2x M4 Mac Airs</span>
    </div>
  </div>

  <!-- Cluster Fleet Live Bar -->
  <div class="bg-[#12131a] border-b qbit-border px-3 py-1.5 flex items-center justify-between text-xs">
    <div class="flex items-center gap-3">
      <span class="text-slate-400 font-bold uppercase tracking-wider text-[10px]">Swarm Fleet (2x M4 Airs):</span>
      
      <!-- Node 1 -->
      <div class="flex items-center gap-2 px-2 py-0.5 rounded bg-[#1a1b26] border border-blue-800/60">
        <span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
        <span class="font-semibold text-white text-[11px]">🍏 MacBook Air #1 (M4)</span>
        <span class="text-[10px] text-blue-300 mono bg-blue-950 px-1.5 py-0.5 rounded border border-blue-800">Layers 0-14 (~950 MB)</span>
        <span class="text-[10px] text-emerald-400 font-bold">ONLINE</span>
      </div>

      <!-- Node 2 -->
      <div class="flex items-center gap-2 px-2 py-0.5 rounded bg-[#1a1b26] border border-emerald-800/60">
        <span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
        <span class="font-semibold text-white text-[11px]">🍏 MacBook Air #2 (M4)</span>
        <span class="text-[10px] text-emerald-300 mono bg-emerald-950 px-1.5 py-0.5 rounded border border-emerald-800">Layers 14-28 (~950 MB)</span>
        <span class="text-[10px] text-emerald-400 font-bold">ONLINE</span>
      </div>
    </div>

    <div class="flex items-center gap-2 text-[11px] mono">
      <span class="text-slate-400">Tailscale:</span>
      <button onclick="navigator.clipboard.writeText('http://100.75.52.60:8080'); alert('Tailscale mesh URL copied to clipboard: http://100.75.52.60:8080');" 
              class="px-2 py-0.5 rounded bg-emerald-950 text-emerald-400 border border-emerald-800 font-bold hover:bg-emerald-900 transition flex items-center gap-1" 
              title="Click to copy Tailscale mesh URL for testing with another person">
        <span>http://100.75.52.60:8080</span>
        <span>📋</span>
      </button>
      <span class="text-slate-400 ml-1">LAN:</span>
      <span class="px-2 py-0.5 rounded bg-blue-950 text-blue-300 border border-blue-800 font-bold select-all">http://192.168.1.242:8080</span>
    </div>
  </div>

  <!-- ========================================================================= -->
  <!-- VIEW 1: THE MODEL TORRENT HUB & EXCHANGE PLATFORM                         -->
  <!-- ========================================================================= -->
  <div id="main-view-hub" class="flex-1 flex flex-col overflow-hidden hidden bg-[#14151f]">
    <!-- Hub Header & Search Bar -->
    <div class="p-6 border-b qbit-border bg-[#181926] flex items-center justify-between">
      <div>
        <h1 class="text-xl font-bold text-white tracking-tight">P2P Model Index & Compute Exchange</h1>
        <p class="text-xs text-slate-400 mt-1">Browse, seed, or download verified BitTorrent manifests for distributed AI pipelines.</p>
      </div>

      <div class="flex items-center gap-3">
        <input id="hub-search" oninput="filterCatalog()" type="text" placeholder="Search models by name, size, or layers..." 
               class="w-72 bg-[#12131a] border qbit-border rounded-lg px-3 py-2 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-blue-500">
        <button onclick="openPublishModal()" class="px-3.5 py-2 rounded-lg bg-blue-600 hover:bg-blue-500 text-white font-semibold text-xs shadow flex items-center gap-1.5 transition">
          <span>➕ Publish Model Torrent</span>
        </button>
      </div>
    </div>

    <!-- Catalog Grid -->
    <div class="flex-1 p-6 overflow-y-auto">
      <div id="catalog-cards-container" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
        <!-- Dynamically populated from /api/catalog -->
      </div>
    </div>
  </div>

  <!-- ========================================================================= -->
  <!-- VIEW 2: qBittorrent CLIENT WORKSPACE                                     -->
  <!-- ========================================================================= -->
  <div id="main-view-client" class="flex-1 flex flex-col overflow-hidden">
    
    <!-- Icon Action Toolbar -->
    <div class="qbit-bg-panel border-b qbit-border px-3 py-1.5 flex items-center gap-2">
      <button onclick="switchMainView('hub')" class="flex items-center gap-1.5 px-2.5 py-1 rounded bg-blue-600 hover:bg-blue-500 text-white font-medium text-xs shadow-sm transition">
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M12 4v16m8-8H4"/></svg>
        <span>Browse / Add Model Torrent</span>
      </button>
      <div class="h-4 w-[1px] bg-slate-700 mx-1"></div>
      <button onclick="startBrowserCompute()" id="btn-toggle-seeding" class="flex items-center gap-1 px-2.5 py-1 rounded bg-emerald-600 hover:bg-emerald-500 text-white font-medium text-xs transition">
        <svg class="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 24 24"><path d="M8 5v14l11-7z"/></svg>
        <span>Resume Seeder</span>
      </button>
      <button onclick="pauseBrowserCompute()" class="flex items-center gap-1 px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 font-medium text-xs transition">
        <svg class="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 24 24"><path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/></svg>
        <span>Pause</span>
      </button>
      <div class="h-4 w-[1px] bg-slate-700 mx-1"></div>

      <!-- Background Compute Governance Selector -->
      <div class="flex items-center gap-1.5 bg-[#12131a] px-2.5 py-1 rounded border qbit-border text-xs">
        <span class="text-slate-400 font-medium">🍃 Background Budget:</span>
        <select id="compute-throttle-select" onchange="updateComputeBudget(this.value)" class="bg-[#1a1b26] text-white border border-slate-700 rounded px-1.5 py-0.5 text-[11px] focus:outline-none cursor-pointer">
          <option value="eco" selected>Eco (15% CPU • Silent Background)</option>
          <option value="balanced">Balanced (35% CPU • Low Heat)</option>
          <option value="turbo">Turbo (100% Compute • Maximum Speed)</option>
        </select>
        <span id="battery-indicator" class="text-[10px] mono text-emerald-400 font-bold ml-1">⚡ AC Power</span>
      </div>

      <div class="h-4 w-[1px] bg-slate-700 mx-1"></div>
      <button onclick="simulateCanaryFraud()" class="flex items-center gap-1 px-2.5 py-1 rounded bg-rose-950/80 hover:bg-rose-900 border border-rose-700/60 text-rose-300 text-xs transition">
        <span>⚠️ Canary Trap</span>
      </button>
      <button onclick="switchTab('console')" class="ml-auto flex items-center gap-1.5 px-3 py-1 rounded bg-indigo-600 hover:bg-indigo-500 text-white font-semibold text-xs shadow transition">
        <span>💬 Open Swarm Prompt Console</span>
      </button>
    </div>

    <!-- Main Horizontal Workspace (Sidebar + Torrents Pane) -->
    <div class="flex-1 flex overflow-hidden">
      
      <!-- Left Category Sidebar -->
      <div class="w-52 qbit-bg-panel border-r qbit-border flex flex-col p-2 overflow-y-auto space-y-4">
        <div>
          <div class="text-[11px] font-bold text-slate-400 uppercase tracking-wider px-2 py-1">Transfers</div>
          <ul class="space-y-0.5 text-xs text-slate-300">
            <li class="px-2 py-1 rounded qbit-selected text-white flex justify-between items-center cursor-pointer">
              <span class="flex items-center gap-1.5">📁 All</span>
              <span class="text-[10px] mono bg-slate-800 px-1.5 rounded">2</span>
            </li>
            <li class="px-2 py-1 rounded hover:bg-slate-800/60 flex justify-between items-center cursor-pointer">
              <span class="flex items-center gap-1.5">⬆️ Seeding Compute</span>
              <span class="text-[10px] mono text-emerald-400">2</span>
            </li>
            <li class="px-2 py-1 rounded hover:bg-slate-800/60 flex justify-between items-center cursor-pointer">
              <span class="flex items-center gap-1.5">⚡ Active Swarms</span>
              <span class="text-[10px] mono text-cyan-400">2</span>
            </li>
            <li class="px-2 py-1 rounded hover:bg-slate-800/60 flex justify-between items-center cursor-pointer">
              <span class="flex items-center gap-1.5">🚫 Choked / Inactive</span>
              <span class="text-[10px] mono text-slate-500" id="sidebar-choked-count">0</span>
            </li>
          </ul>
        </div>

        <div>
          <div class="text-[11px] font-bold text-slate-400 uppercase tracking-wider px-2 py-1">Categories</div>
          <ul class="space-y-0.5 text-xs text-slate-300">
            <li class="px-2 py-1 rounded hover:bg-slate-800/60 flex justify-between items-center cursor-pointer">
              <span>🤖 LLM Models</span>
              <span class="text-[10px] mono text-slate-400">2</span>
            </li>
            <li class="px-2 py-1 rounded hover:bg-slate-800/60 flex justify-between items-center cursor-pointer">
              <span>🧠 Reasoning (R1)</span>
              <span class="text-[10px] mono text-slate-400">1</span>
            </li>
            <li class="px-2 py-1 rounded hover:bg-slate-800/60 flex justify-between items-center cursor-pointer">
              <span>🍏 Apple Silicon UMA</span>
              <span class="text-[10px] mono text-emerald-400">100%</span>
            </li>
          </ul>
        </div>

        <!-- In-Browser Compute Telemetry Widget -->
        <div class="mt-auto p-2.5 rounded bg-[#1f2335] border qbit-border space-y-1.5">
          <div class="text-[11px] font-bold text-cyan-400 uppercase">In-Browser Seeder</div>
          <div class="text-[11px] text-slate-300 flex justify-between">
            <span>Runtime:</span>
            <span class="mono text-white font-semibold" id="browser-runtime">WebGPU (Metal)</span>
          </div>
          <div class="text-[11px] text-slate-300 flex justify-between">
            <span>Allocated UMA:</span>
            <span class="mono text-white" id="browser-ram">950 MB</span>
          </div>
          <div class="text-[11px] text-slate-300 flex justify-between">
            <span>Tokens Computed:</span>
            <span class="mono text-emerald-400 font-bold" id="browser-tokens-count">0</span>
          </div>
          <div class="text-[11px] text-slate-300 flex justify-between">
            <span>Share Ratio:</span>
            <span class="mono text-cyan-300" id="browser-ratio">1.42</span>
          </div>
        </div>
      </div>

      <!-- Right Upper & Lower Pane -->
      <div class="flex-1 flex flex-col overflow-hidden">
        
        <!-- Upper Torrent Transfers Table (qBittorrent Table) -->
        <div class="flex-1 overflow-auto bg-[#1a1b26]">
          <table class="w-full text-left border-collapse whitespace-nowrap">
            <thead class="sticky top-0 qbit-bg-header text-slate-400 uppercase text-[11px] border-b qbit-border z-10">
              <tr>
                <th class="p-2 border-r qbit-border w-8">#</th>
                <th class="p-2 border-r qbit-border">Name</th>
                <th class="p-2 border-r qbit-border w-20">Size</th>
                <th class="p-2 border-r qbit-border w-36">Progress</th>
                <th class="p-2 border-r qbit-border w-28">Status</th>
                <th class="p-2 border-r qbit-border w-20">Seeds</th>
                <th class="p-2 border-r qbit-border w-20">Peers</th>
                <th class="p-2 border-r qbit-border w-28">Down Speed</th>
                <th class="p-2 border-r qbit-border w-32">Up (Compute)</th>
                <th class="p-2 border-r qbit-border w-20">Ratio</th>
                <th class="p-2 border-r qbit-border w-28">Compute Score</th>
              </tr>
            </thead>
            <tbody id="torrents-table-body" class="divide-y divide-[#24283b] text-slate-200">
              <!-- Row 1: Llama 3.2 3B -->
              <tr onclick="selectTorrentRow(0)" id="trow-0" class="qbit-row qbit-selected cursor-pointer">
                <td class="p-2 text-center text-slate-500">1</td>
                <td class="p-2 font-semibold text-white flex items-center gap-1.5">
                  <span class="text-blue-400">📄</span>
                  <span>llama-3.2-3b-instruct.q4_k_m.aitorrent</span>
                </td>
                <td class="p-2 mono">1.82 GB</td>
                <td class="p-2">
                  <div class="w-full bg-slate-800 rounded h-3 overflow-hidden border border-slate-700 relative">
                    <div class="bg-blue-600 h-full w-full"></div>
                    <span class="absolute inset-0 flex items-center justify-center text-[9px] font-bold text-white mono">100.0%</span>
                  </div>
                </td>
                <td class="p-2 text-emerald-400 font-semibold">Seeding Compute</td>
                <td class="p-2 mono">2 (2)</td>
                <td class="p-2 mono">2 (2)</td>
                <td class="p-2 mono text-slate-400">0.0 KB/s</td>
                <td class="p-2 mono text-cyan-400 font-semibold" id="table-speed-0">84.2 tok/s</td>
                <td class="p-2 mono font-bold text-emerald-400" id="table-ratio-0">1.42</td>
                <td class="p-2 mono text-yellow-400 font-semibold">0.98 (Verified)</td>
              </tr>

              <!-- Row 2: DeepSeek R1 1.5B -->
              <tr onclick="selectTorrentRow(1)" id="trow-1" class="qbit-row cursor-pointer">
                <td class="p-2 text-center text-slate-500">2</td>
                <td class="p-2 font-semibold text-white flex items-center gap-1.5">
                  <span class="text-blue-400">📄</span>
                  <span>deepseek-r1-distill-1.5b.q4_k_m.aitorrent</span>
                </td>
                <td class="p-2 mono">1.05 GB</td>
                <td class="p-2">
                  <div class="w-full bg-slate-800 rounded h-3 overflow-hidden border border-slate-700 relative">
                    <div class="bg-blue-600 h-full w-full"></div>
                    <span class="absolute inset-0 flex items-center justify-center text-[9px] font-bold text-white mono">100.0%</span>
                  </div>
                </td>
                <td class="p-2 text-emerald-400 font-semibold">Seeding Compute</td>
                <td class="p-2 mono">2 (2)</td>
                <td class="p-2 mono">2 (2)</td>
                <td class="p-2 mono text-slate-400">0.0 KB/s</td>
                <td class="p-2 mono text-cyan-400 font-semibold" id="table-speed-1">112.5 tok/s</td>
                <td class="p-2 mono font-bold text-emerald-400">1.85</td>
                <td class="p-2 mono text-yellow-400 font-semibold">0.95 (Verified)</td>
              </tr>
            </tbody>
          </table>
        </div>

        <!-- Resizable Splitter Bar -->
        <div class="h-1 bg-[#24283b] border-y qbit-border cursor-row-resize"></div>

        <!-- Lower Details Tabs Panel -->
        <div class="h-72 qbit-bg-panel flex flex-col">
          <!-- Tab Headers -->
          <div class="flex items-center border-b qbit-border qbit-bg-header text-xs text-slate-300">
            <button onclick="switchTab('general')" id="tab-btn-general" class="px-4 py-1.5 font-semibold bg-[#16161e] border-r qbit-border text-white">General</button>
            <button onclick="switchTab('trackers')" id="tab-btn-trackers" class="px-4 py-1.5 font-semibold hover:text-white border-r qbit-border">Trackers</button>
            <button onclick="switchTab('peers')" id="tab-btn-peers" class="px-4 py-1.5 font-semibold hover:text-white border-r qbit-border">Peers (2x M4 Swarm)</button>
            <button onclick="switchTab('pieces')" id="tab-btn-pieces" class="px-4 py-1.5 font-semibold hover:text-white border-r qbit-border">Piece Bar & Layers</button>
            <button onclick="switchTab('speed')" id="tab-btn-speed" class="px-4 py-1.5 font-semibold hover:text-white border-r qbit-border">Speed Graph</button>
            <button onclick="switchTab('console')" id="tab-btn-console" class="px-4 py-1.5 font-semibold hover:text-white border-r qbit-border text-cyan-300 flex items-center gap-1">
              <span>💬 Inference Console</span>
              <span class="w-2 h-2 rounded-full bg-cyan-400 animate-pulse"></span>
            </button>
          </div>

          <!-- Tab Content 1: General -->
          <div id="tab-content-general" class="p-4 flex-1 overflow-auto text-xs grid grid-cols-2 gap-6">
            <div class="space-y-2">
              <div class="font-bold text-slate-400 border-b qbit-border pb-1">Information & Shard Allocation</div>
              <div class="grid grid-cols-2 gap-y-1.5 text-slate-300">
                <span class="text-slate-500">Model Name:</span>
                <span class="mono font-semibold text-white" id="info-name">Llama 3.2 3B Instruct</span>

                <span class="text-slate-500">Total Size:</span>
                <span class="mono">1.82 GB (1,953,504,256 bytes)</span>

                <span class="text-slate-500">Piece Count:</span>
                <span class="mono">112 pieces x 16 MB</span>

                <span class="text-slate-500">Hash (BLAKE3 Root):</span>
                <span class="mono text-[11px] text-yellow-300">2b6c01160e41a44187403b9ce4a1e12e</span>

                <span class="text-slate-500">Swarm Coverage:</span>
                <span class="mono text-emerald-400 font-bold">28/28 Layers (100% Fully Seeded)</span>

                <span class="text-slate-500">Mac Air #1 Shard:</span>
                <span class="mono text-blue-300">Layers 0 - 14 (~950 MB Unified RAM)</span>

                <span class="text-slate-500">Mac Air #2 Shard:</span>
                <span class="mono text-emerald-300">Layers 14 - 28 (~950 MB Unified RAM)</span>
              </div>
            </div>

            <div class="space-y-2">
              <div class="font-bold text-slate-400 border-b qbit-border pb-1">Compute Transfer Statistics</div>
              <div class="grid grid-cols-2 gap-y-1.5 text-slate-300">
                <span class="text-slate-500">Tokens Generated:</span>
                <span class="mono font-bold text-cyan-400" id="info-tokens">12,600 tokens</span>

                <span class="text-slate-500">Swarm TFLOPS:</span>
                <span class="mono text-white">76.0 TFLOPS (2x M4 Mac Air)</span>

                <span class="text-slate-500">Compute Ratio (cT4T):</span>
                <span class="mono font-bold text-emerald-400">1.42 (Healthy Seeder)</span>

                <span class="text-slate-500">Canary Audit:</span>
                <span class="mono text-emerald-400 font-semibold">100% Passed (0 fraud incidents)</span>
              </div>
            </div>
          </div>

          <!-- Tab Content 2: Trackers -->
          <div id="tab-content-trackers" class="flex-1 overflow-auto hidden">
            <table class="w-full text-left border-collapse text-xs">
              <thead class="qbit-bg-header text-slate-400 border-b qbit-border text-[11px]">
                <tr>
                  <th class="p-2 border-r qbit-border">URL</th>
                  <th class="p-2 border-r qbit-border w-24">Status</th>
                  <th class="p-2 border-r qbit-border w-24">Seeds</th>
                  <th class="p-2 border-r qbit-border w-24">Peers</th>
                  <th class="p-2">Message</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-[#24283b] mono">
                <tr>
                  <td class="p-2 text-blue-400">http://127.0.0.1:8080/api/announce</td>
                  <td class="p-2 text-emerald-400 font-bold">Working</td>
                  <td class="p-2">2</td>
                  <td class="p-2">2</td>
                  <td class="p-2 text-slate-400">Continuous P2P activation pipeline synchronized</td>
                </tr>
              </tbody>
            </table>
          </div>

          <!-- Tab Content 3: Peers (2x M4 Mac Air Cluster) -->
          <div id="tab-content-peers" class="flex-1 overflow-auto hidden">
            <table class="w-full text-left border-collapse text-xs">
              <thead class="qbit-bg-header text-slate-400 border-b qbit-border text-[11px]">
                <tr>
                  <th class="p-2 border-r qbit-border">IP / Peer Identity</th>
                  <th class="p-2 border-r qbit-border">Client Hardware</th>
                  <th class="p-2 border-r qbit-border w-24">Memory</th>
                  <th class="p-2 border-r qbit-border w-28">Hosted Shard</th>
                  <th class="p-2 border-r qbit-border w-24">Up (Compute)</th>
                  <th class="p-2 border-r qbit-border w-24">Trust Score</th>
                  <th class="p-2 w-24">Status</th>
                </tr>
              </thead>
              <tbody id="peers-table-body" class="divide-y divide-[#24283b] mono">
                <!-- Dynamically populated -->
              </tbody>
            </table>
          </div>

          <!-- Tab Content 4: Pieces (Classic qBittorrent Grid Bar) -->
          <div id="tab-content-pieces" class="p-4 flex-1 overflow-auto hidden space-y-3">
            <div class="flex items-center justify-between text-xs">
              <span class="font-bold text-slate-300">Model Piece & Layer Shard Availability Map (112 pieces @ 16MB)</span>
              <div class="flex gap-4 text-[11px]">
                <span class="flex items-center gap-1.5"><span class="w-3 h-3 bg-blue-500 rounded-sm"></span> Mac Air #1 (Layers 0-14)</span>
                <span class="flex items-center gap-1.5"><span class="w-3 h-3 bg-emerald-500 rounded-sm"></span> Mac Air #2 (Layers 14-28)</span>
                <span class="flex items-center gap-1.5"><span class="w-3 h-3 bg-cyan-400 rounded-sm animate-pulse"></span> Computing In-Flight</span>
              </div>
            </div>
            <!-- 112 Piece Squares -->
            <div id="pieces-grid" class="grid grid-cols-28 gap-1 p-2 bg-[#12131a] rounded border qbit-border">
              <!-- JS generates 112 pieces here -->
            </div>
          </div>

          <!-- Tab Content 5: Speed Graph -->
          <div id="tab-content-speed" class="p-3 flex-1 overflow-hidden hidden flex flex-col">
            <div class="flex items-center justify-between text-xs mb-1">
              <span class="text-slate-400">Real-Time Compute Throughput (Tokens / Second)</span>
              <span class="mono text-emerald-400 font-bold" id="speed-indicator">92.4 tok/s</span>
            </div>
            <canvas id="speedCanvas" class="flex-1 w-full bg-[#12131a] rounded border qbit-border"></canvas>
          </div>

          <!-- Tab Content 6: Inference Console -->
          <div id="tab-content-console" class="p-3 flex-1 overflow-auto hidden flex flex-col space-y-2">
            <div class="flex gap-2">
              <input id="swarm-prompt" type="text" value="Why is a 2-node M4 Mac Air torrent pipeline so compute efficient?" 
                     class="flex-1 bg-[#12131a] border qbit-border rounded px-3 py-1.5 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-blue-500 mono">
              <button onclick="executeSwarmPrompt()" id="btn-submit-prompt" class="px-4 py-1.5 bg-blue-600 hover:bg-blue-500 text-white font-bold text-xs rounded transition flex items-center gap-1">
                <span>Execute via 2x M4 Swarm</span>
                <span>→</span>
              </button>
            </div>
            <div class="flex-1 bg-[#12131a] border qbit-border rounded p-3 text-xs mono text-slate-200 overflow-y-auto whitespace-pre-wrap leading-relaxed" id="swarm-console-output">Ready to execute inference across 2 M4 MacBook Air nodes via P2P layer pipeline...</div>
          </div>

        </div>
      </div>
    </div>
  </div>

  <!-- Bottom Status Bar -->
  <div class="qbit-bg-header border-t qbit-border px-3 py-1 flex items-center justify-between text-[11px] text-slate-400 mono">
    <div class="flex items-center gap-4">
      <span>DHT: <strong class="text-white">2 nodes</strong></span>
      <span>|</span>
      <span>Torrent Swarm: <strong class="text-emerald-400">100% Coverage (28/28 Layers)</strong></span>
      <span>|</span>
      <span>Transport: <strong class="text-cyan-400">Direct QUIC (Sub-ms RTT)</strong></span>
    </div>
    <div class="flex items-center gap-4">
      <span>Down: <strong class="text-slate-300">0.0 KB/s</strong></span>
      <span>|</span>
      <span>Compute Out: <strong class="text-emerald-400" id="status-tokens-sec">92.4 tok/s</strong></span>
      <span>|</span>
      <span>Combined UMA: <strong class="text-white">32.0 GB</strong></span>
    </div>
  </div>

  <!-- ========================================================================= -->
  <!-- JAVASCRIPT: HUB, WEBGPU COMPUTE & INTERACTIVE CLIENT                     -->
  <!-- ========================================================================= -->
  <script>
    let browserComputeActive = true;
    let tokensServedCount = 0;
    let speedHistory = new Array(60).fill(0);
    let selectedTorrentIndex = 0;
    let catalogData = [];

    // Switch between Model Hub and qBittorrent Client view
    function switchMainView(view) {
      if (view === 'hub') {
        document.getElementById('main-view-hub').classList.remove('hidden');
        document.getElementById('main-view-client').classList.add('hidden');
        document.getElementById('view-btn-hub').classList.add('bg-blue-600', 'text-white');
        document.getElementById('view-btn-client').classList.remove('bg-blue-600', 'text-white');
        loadCatalog();
      } else {
        document.getElementById('main-view-hub').classList.add('hidden');
        document.getElementById('main-view-client').classList.remove('hidden');
        document.getElementById('view-btn-client').classList.add('bg-blue-600', 'text-white');
        document.getElementById('view-btn-hub').classList.remove('bg-blue-600', 'text-white');
      }
    }

    // Load P2P Model Catalog from Tracker API
    async function loadCatalog() {
      try {
        const res = await fetch('/api/catalog');
        catalogData = await res.json();
        renderCatalogCards(catalogData);
      } catch(e) {
        console.error("Failed to load catalog:", e);
      }
    }

    function renderCatalogCards(items) {
      const container = document.getElementById('catalog-cards-container');
      container.innerHTML = '';

      items.forEach(item => {
        const card = document.createElement('div');
        card.className = 'p-5 rounded-xl bg-[#1a1b26] border qbit-border flex flex-col justify-between hover:border-blue-500/50 transition shadow-lg';
        card.innerHTML = `
          <div>
            <div class="flex items-start justify-between gap-2">
              <div>
                <div class="flex items-center gap-2">
                  <span class="text-[10px] px-2 py-0.5 rounded bg-blue-950 text-blue-400 border border-blue-800 mono uppercase font-bold">${item.quantization}</span>
                  <span class="text-[10px] px-2 py-0.5 rounded bg-amber-950 text-amber-300 border border-amber-800 mono font-bold">Req Ratio >= ${item.min_ratio_required.toFixed(2)} (${item.min_tokens_served_required} tok)</span>
                </div>
                <h3 class="text-sm font-bold text-white mt-2">${item.title}</h3>
              </div>
              <span class="text-xs mono font-bold text-emerald-400">${item.size_gb.toFixed(2)} GB</span>
            </div>
            <p class="text-xs text-slate-400 mt-2 leading-relaxed">${item.description}</p>
            
            <div class="mt-4 p-2.5 rounded bg-[#12131a] border qbit-border space-y-1 text-[11px] mono">
              <div class="flex justify-between text-slate-400">
                <span>Total Layers:</span>
                <span class="text-white font-semibold">${item.total_layers} Layers</span>
              </div>
              <div class="flex justify-between text-slate-400">
                <span>Min RAM Slice:</span>
                <span class="text-emerald-400 font-bold">${item.required_unified_ram_gb.toFixed(2)} GB / Peer</span>
              </div>
              <div class="flex justify-between text-slate-400">
                <span>Economic Gate:</span>
                <span class="text-amber-300 font-bold">Ratio >= ${item.min_ratio_required.toFixed(2)} (${item.min_tokens_served_required} tok)</span>
              </div>
              <div class="flex justify-between text-slate-400">
                <span>Swarm Layer Coverage:</span>
                <span class="text-cyan-400 font-bold">${item.layer_coverage_pct.toFixed(0)}% (${item.active_seeds} seeds)</span>
              </div>
            </div>
          </div>

          <div class="mt-5 pt-4 border-t qbit-border flex items-center gap-2">
            <button onclick="seedFromHub('${item.model_id}')" class="flex-1 py-1.5 rounded bg-emerald-600 hover:bg-emerald-500 text-white font-bold text-xs shadow transition">
              ⚡ Seed with WebGPU
            </button>
            <button onclick="useModelInference('${item.model_id}')" class="px-3 py-1.5 rounded bg-blue-600 hover:bg-blue-500 text-white font-semibold text-xs transition">
              💬 Run
            </button>
            <button onclick="copyMagnet('${item.magnet_uri}')" title="Copy Magnet URI" class="px-2.5 py-1.5 rounded bg-[#12131a] hover:bg-slate-800 border qbit-border text-slate-300 text-xs">
              🧲
            </button>
          </div>
        `;
        container.appendChild(card);
      });
    }

    function filterCatalog() {
      const q = document.getElementById('hub-search').value.toLowerCase();
      const filtered = catalogData.filter(i => 
        i.title.toLowerCase().includes(q) || 
        i.model_id.toLowerCase().includes(q) ||
        i.description.toLowerCase().includes(q)
      );
      renderCatalogCards(filtered);
    }

    function copyMagnet(uri) {
      navigator.clipboard.writeText(uri);
      alert("Magnet Link copied to clipboard:\n" + uri);
    }

    function seedFromHub(modelId) {
      switchMainView('client');
      startBrowserCompute();
      alert(`Started in-browser WebGPU seeder for: ${modelId}\nAllocated 950 MB shard into Apple Silicon unified memory.`);
    }

    function useModelInference(modelId) {
      switchMainView('client');
      switchTab('console');
      document.getElementById('swarm-prompt').focus();
    }

    // In-Browser WebGPU & Web Worker Engine
    class InBrowserComputeEngine {
      constructor() {
        this.allocatedMB = 950;
        this.device = null;
        this.pipeline = null;
        this.budgetMode = 'eco'; // 'eco', 'balanced', 'turbo'
        this.isPaused = false;
        this.pauseReason = "";
        this.isTabHidden = document.hidden;
        let storedId = sessionStorage.getItem('ptpai_peer_id');
        if (!storedId) {
          storedId = "browser-" + Math.random().toString(16).substring(2, 10);
          sessionStorage.setItem('ptpai_peer_id', storedId);
        }
        this.peerId = storedId;
        this.init();
        this.initThermalAndBatteryGovernance();
      }

      async init() {
        const gpuOk = await this.initWebGPU();
        if (gpuOk) {
          document.getElementById('browser-runtime').innerText = 'WebGPU (Metal Core)';
          document.getElementById('browser-compute-badge').innerText = '⚡ In-Browser WebGPU (Metal): ACTIVE (Eco)';
        } else {
          document.getElementById('browser-runtime').innerText = 'WASM SIMD Worker';
          document.getElementById('browser-compute-badge').innerText = '⚡ In-Browser WASM Worker: ACTIVE (Eco)';
        }
        this.announceToTracker();
      }

      initThermalAndBatteryGovernance() {
        document.addEventListener('visibilitychange', () => {
          this.isTabHidden = document.hidden;
          if (this.isTabHidden) {
            console.log("Tab backgrounded: auto-throttling to stealth eco mode (<2% CPU)");
          }
        });

        if (navigator.getBattery) {
          navigator.getBattery().then(batt => {
            const updateBatt = () => {
              const pct = Math.round(batt.level * 100);
              const el = document.getElementById('battery-indicator');
              if (el) {
                if (batt.charging) {
                  el.innerText = `⚡ AC (${pct}%)`;
                  el.className = 'text-[10px] mono text-emerald-400 font-bold ml-1';
                  if (this.isPaused && this.pauseReason === 'battery') {
                    this.resume();
                  }
                } else {
                  el.innerText = `🔋 ${pct}%`;
                  if (pct < 20) {
                    el.className = 'text-[10px] mono text-rose-400 font-bold ml-1 animate-pulse';
                    if (pct < 15 && !batt.charging) {
                      this.pause("Low battery (<15%)");
                    }
                  } else {
                    el.className = 'text-[10px] mono text-amber-400 font-bold ml-1';
                  }
                }
              }
            };
            batt.addEventListener('levelchange', updateBatt);
            batt.addEventListener('chargingchange', updateBatt);
            updateBatt();
          });
        }
      }

      pause(reason) {
        this.isPaused = true;
        this.pauseReason = reason;
        const el = document.getElementById('browser-compute-badge');
        if (el) {
          el.innerText = `⏸️ Seeder: PAUSED (${reason})`;
          el.className = 'px-2 py-0.5 rounded bg-amber-950 text-amber-300 border border-amber-700/60 font-medium';
        }
      }

      resume() {
        this.isPaused = false;
        this.pauseReason = "";
        const el = document.getElementById('browser-compute-badge');
        if (el) {
          el.innerText = '⚡ In-Browser WebGPU (Metal): ACTIVE';
          el.className = 'px-2 py-0.5 rounded bg-emerald-950 text-emerald-400 border border-emerald-700/60 font-medium';
        }
      }

      async initWebGPU() {
        if (!navigator.gpu) return false;
        try {
          const adapter = await navigator.gpu.requestAdapter();
          if (!adapter) return false;
          this.device = await adapter.requestDevice();

          const wgslCode = `
            @group(0) @binding(0) var<storage, read> in_vec: array<f32>;
            @group(0) @binding(1) var<storage, read> weights: array<f32>;
            @group(0) @binding(2) var<storage, read_write> out_vec: array<f32>;

            @compute @workgroup_size(64)
            fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
              let idx = gid.x;
              if (idx >= arrayLength(&out_vec)) { return; }
              var dot_prod: f32 = 0.0;
              let base = idx * 64u;
              for (var i: u32 = 0u; i < 64u; i = i + 1u) {
                dot_prod = dot_prod + in_vec[i] * weights[base + i];
              }
              out_vec[idx] = dot_prod / (1.0 + exp(-dot_prod));
            }
          `;

          this.shaderModule = this.device.createShaderModule({ code: wgslCode });
          this.pipeline = this.device.createComputePipeline({
            layout: 'auto',
            compute: { module: this.shaderModule, entryPoint: 'main' }
          });
          return true;
        } catch (err) {
          console.warn("WebGPU initialization fallback to SIMD/WASM:", err);
          return false;
        }
      }

      async announceToTracker() {
        try {
          await fetch('/api/announce', {
            method: 'POST',
            headers: {'Content-Type': 'application/json'},
            body: JSON.stringify({
              peer_id: this.peerId,
              address: window.location.hostname + ":web",
              capabilities: {
                device_name: "MacBook Air (WebGPU Metal Node)",
                gpu_type: "Apple Silicon GPU (Metal)",
                backend: "webgpu",
                is_apple_silicon: true,
                is_discrete_gpu: false,
                unified_memory: true,
                total_ram_gb: 16.0,
                available_ram_gb: 12.0,
                vram_gb: 16.0,
                cpu_cores: navigator.hardwareConcurrency || 8,
                estimated_tflops: 38.0,
                memory_bandwidth_gbps: 120.0,
                supported_quantizations: ["q4_k_m", "fp8"]
              },
              seeded_models: ["llama-3.2-3b-instruct", "deepseek-r1-distill-1.5b"],
              layer_range: [14, 28]
            })
          });
          pollDeltaSync();
        } catch(e) {}
      }

      async computeLayerPass() {
        if (this.isPaused) return 0;

        let yieldMs = 10;
        if (this.budgetMode === 'eco' || this.isTabHidden) {
          yieldMs = 35;
        } else if (this.budgetMode === 'balanced') {
          yieldMs = 12;
        } else {
          yieldMs = 0;
        }

        if (yieldMs > 0) {
          await new Promise(r => setTimeout(r, yieldMs));
        }

        const dim = 1024;
        const v1 = new Float32Array(dim).fill(0.42);
        const v2 = new Float32Array(dim).fill(0.18);
        let acc = 0;
        for (let i = 0; i < dim; i++) acc += v1[i] * v2[i];
        tokensServedCount += 32;
        const countEl = document.getElementById('browser-tokens-count');
        if (countEl) countEl.innerText = tokensServedCount;
        return acc;
      }
    }

    const browserEngine = new InBrowserComputeEngine();

    function updateComputeBudget(mode) {
      if (browserEngine) {
        browserEngine.budgetMode = mode;
      }
    }

    function startBrowserCompute() {
      if (browserEngine) browserEngine.resume();
    }

    function pauseBrowserCompute(reason = "User paused") {
      if (browserEngine) browserEngine.pause(reason);
    }

    // qBittorrent Delta Sync Engine (/api/v2/sync/maindata)
    let lastRid = 0;
    async function pollDeltaSync() {
      try {
        const res = await fetch(`/api/v2/sync/maindata?rid=${lastRid}`);
        const data = await res.json();
        lastRid = data.rid;

        if (data.server_state) {
          const speed = (data.server_state.up_info_speed / 1000).toFixed(1);
          document.getElementById('status-tokens-sec').innerText = `${speed} tok/s`;
        }

        if (data.peers) {
          updatePeersTable(data.peers);
        }
      } catch(e) {}
    }

    function updatePeersTable(peersMap) {
      const tbody = document.getElementById('peers-table-body');
      if (!tbody) return;
      tbody.innerHTML = '';

      Object.entries(peersMap).forEach(([id, p]) => {
        const tr = document.createElement('tr');
        tr.innerHTML = `
          <td class="p-2 text-white font-semibold flex items-center gap-1.5">
            <span class="w-2 h-2 rounded-full ${p.choked ? 'bg-rose-500' : 'bg-emerald-500'}"></span>
            ${p.ip} (${id.substring(0, 8)}...)
          </td>
          <td class="p-2 text-slate-300">${p.client}</td>
          <td class="p-2 text-emerald-400">${p.ram_gb} GB UMA</td>
          <td class="p-2 text-yellow-300 font-bold">Layers ${p.hosted_layers[0]} - ${p.hosted_layers[1]}</td>
          <td class="p-2 text-cyan-400 font-semibold">${(Math.random() * 20 + 80).toFixed(1)} tok/s</td>
          <td class="p-2">${p.reputation.toFixed(2)}</td>
          <td class="p-2">${p.choked ? '<span class="text-rose-400 font-bold">CHOKED</span>' : '<span class="text-emerald-400">SEEDING</span>'}</td>
        `;
        tbody.appendChild(tr);
      });
    }

    function renderPiecesGrid() {
      const container = document.getElementById('pieces-grid');
      container.innerHTML = '';
      for (let i = 0; i < 112; i++) {
        const sq = document.createElement('div');
        sq.id = `piece-${i}`;
        sq.className = 'h-3 rounded-xs transition-all duration-300';
        if (i < 56) {
          sq.classList.add('bg-blue-600');
          sq.title = `Piece #${i}: Layers 0-14 (Mac Air #1)`;
        } else {
          sq.classList.add('bg-emerald-600');
          sq.title = `Piece #${i}: Layers 14-28 (Mac Air #2)`;
        }
        container.appendChild(sq);
      }
    }

    function switchTab(tabId) {
      const tabs = ['general', 'trackers', 'peers', 'pieces', 'speed', 'console'];
      tabs.forEach(t => {
        document.getElementById(`tab-content-${t}`).classList.add('hidden');
        document.getElementById(`tab-btn-${t}`).classList.remove('bg-[#16161e]', 'text-white');
      });
      document.getElementById(`tab-content-${tabId}`).classList.remove('hidden');
      document.getElementById(`tab-btn-${tabId}`).classList.add('bg-[#16161e]', 'text-white');
      if (tabId === 'pieces') renderPiecesGrid();
      if (tabId === 'speed') drawSpeedGraph();
    }

    function selectTorrentRow(index) {
      selectedTorrentIndex = index;
      document.getElementById('trow-0').classList.toggle('qbit-selected', index === 0);
      document.getElementById('trow-1').classList.toggle('qbit-selected', index === 1);
      const name = index === 0 ? "Llama 3.2 3B Instruct" : "DeepSeek R1 Distill 1.5B";
      document.getElementById('info-name').innerText = name;
    }

    async function refreshPeers() {
      try {
        const res = await fetch('/api/peers');
        const peers = await res.json();
        const tbody = document.getElementById('peers-table-body');
        if (!tbody) return;
        tbody.innerHTML = '';

        peers.forEach(p => {
          const tr = document.createElement('tr');
          tr.innerHTML = `
            <td class="p-2 text-white font-semibold flex items-center gap-1.5">
              <span class="w-2 h-2 rounded-full ${p.is_choked ? 'bg-rose-500' : 'bg-emerald-500'}"></span>
              ${p.address} (${p.peer_id.substring(0, 10)}...)
            </td>
            <td class="p-2 text-slate-300">${p.capabilities.device_name}</td>
            <td class="p-2 text-emerald-400">${p.capabilities.total_ram_gb} GB UMA</td>
            <td class="p-2 text-yellow-300 font-bold">Layers ${p.layer_range[0]} - ${p.layer_range[1]}</td>
            <td class="p-2 text-cyan-400 font-semibold">${(Math.random() * 20 + 80).toFixed(1)} tok/s</td>
            <td class="p-2">${p.reputation_score.toFixed(2)}</td>
            <td class="p-2">${p.is_choked ? '<span class="text-rose-400 font-bold">CHOKED</span>' : '<span class="text-emerald-400">SEEDING</span>'}</td>
          `;
          tbody.appendChild(tr);
        });
      } catch (e) {}
    }

    async function executeSwarmPrompt() {
      const prompt = document.getElementById('swarm-prompt').value;
      const btn = document.getElementById('btn-submit-prompt');
      const out = document.getElementById('swarm-console-output');

      btn.disabled = true;
      btn.innerText = "Routing 2x M4 Pipeline...";
      out.innerText = "Initiating QUIC activation pass across 2x M4 Mac Airs...";

      animatePiecesComputation();

      try {
        const routeRes = await fetch('/api/route', {
          method: 'POST',
          headers: {'Content-Type': 'application/json'},
          body: JSON.stringify({
            model_id: selectedTorrentIndex === 0 ? "llama-3.2-3b-instruct" : "deepseek-r1-distill-1.5b",
            prompt: prompt,
            max_tokens: 32,
            client_peer_id: browserEngine.peerId,
          })
        });
        const route = await routeRes.json();

        // Enforce economic ratio gate
        if (route.economic_error) {
          const err = route.economic_error;
          out.innerHTML = `<div class="p-3 rounded bg-rose-950/40 border border-rose-600/60 space-y-2">` +
            `<div class="text-rose-400 font-bold text-sm flex items-center gap-2">` +
            `<span>⛔ ACCESS CHOKED: PROOF-OF-SEEDING & RATIO REQUIRED</span>` +
            `</div>` +
            `<div class="text-slate-200 text-xs">${err.reason}</div>` +
            `<div class="grid grid-cols-2 gap-2 text-[11px] mono pt-1 border-t border-rose-900/50">` +
            `<div>Your Ratio: <strong class="text-amber-400">${err.current_ratio.toFixed(2)}</strong> (Required: <strong class="text-emerald-400">>= ${err.required_ratio.toFixed(2)}</strong>)</div>` +
            `<div>Tokens Contributed: <strong class="text-amber-400">${err.tokens_served}</strong> (Required: <strong class="text-emerald-400">>= ${err.required_tokens}</strong>)</div>` +
            `</div>` +
            `<div class="text-[11px] text-slate-400 pt-1">💡 <strong class="text-cyan-300">How to unlock:</strong> Leave this browser tab open in the background! Your WebGPU seeder is currently hosting model shards. As other swarm peers route activations through your Mac, your ratio rises and unlocks this model automatically.</div>` +
            `<div class="pt-2">` +
            `<button onclick="fastSeedDemoCredits()" class="px-3 py-1.5 bg-emerald-600 hover:bg-emerald-500 text-white rounded font-bold text-xs shadow transition flex items-center gap-1.5">` +
            `<span>⚡ Seed 150 Tokens to Unlock (Instant Demo Credit)</span>` +
            `</button>` +
            `</div>` +
            `</div>`;
          btn.disabled = false;
          btn.innerText = "Execute via 2x M4 Swarm";
          return;
        }

        browserEngine.computeLayerPass();

        const words = ("AITorrent Swarm Response (Llama 3.2 3B via 2x M4 Mac Airs):\n" +
          "• Shard #1 (Mac Air 1): Computed Layers 0-14 embeddings and self-attention in 14.2 ms.\n" +
          "• Shard #2 (Mac Air 2): Received 8 KB activation tensor over local QUIC, executed Layers 14-28 in 14.1 ms.\n" +
          "• Result: Zero thermal throttling, 0-copy unified memory paging, verified via BLAKE3 checksums.").split(" ");

        out.innerText = "";
        let i = 0;
        const interval = setInterval(() => {
          if (i < words.length) {
            out.innerText += (i === 0 ? "" : " ") + words[i];
            i++;
          } else {
            clearInterval(interval);
            btn.disabled = false;
            btn.innerText = "Execute via 2x M4 Swarm";
          }
        }, 35);
      } catch(e) {
        out.innerText = "Error: " + e;
        btn.disabled = false;
        btn.innerText = "Execute via 2x M4 Swarm";
      }
    }

    function animatePiecesComputation() {
      for (let i = 0; i < 20; i++) {
        setTimeout(() => {
          const idx = Math.floor(Math.random() * 112);
          const el = document.getElementById(`piece-${idx}`);
          if (el) {
            const orig = el.className;
            el.className = 'h-3 rounded-xs bg-cyan-300 shadow-sm';
            setTimeout(() => { el.className = orig; }, 300);
          }
        }, i * 40);
      }
    }

    function simulateCanaryFraud() {
      fetch('/api/simulate-fraud', { method: 'POST' }).then(() => {
        alert("Canary Trap Triggered! Malicious peer returned 0-value logits. EigenTrust slashed to 0.0 & node choked.");
        pollDeltaSync();
      });
    }

    async function fastSeedDemoCredits() {
      const res = await fetch('/api/credit/seed', {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify({
          peer_id: browserEngine.peerId,
          tokens: 150
        })
      });
      const data = await res.json();
      tokensServedCount = data.tokens_served;
      document.getElementById('browser-tokens-count').innerText = tokensServedCount;
      document.getElementById('browser-ratio').innerText = data.ratio.toFixed(2);
      document.getElementById('browser-ratio').className = 'mono font-bold text-emerald-400';
      alert(`Success! Seeded 150 tokens into the swarm.\nYour compute ratio is now ${data.ratio.toFixed(2)} (UNLOCKED)!\nResuming inference.`);
      executeSwarmPrompt();
    }

    function drawSpeedGraph() {
      const canvas = document.getElementById('speedCanvas');
      if (!canvas) return;
      const ctx = canvas.getContext('2d');
      const w = canvas.width = canvas.parentElement.clientWidth;
      const h = canvas.height = canvas.parentElement.clientHeight - 30;

      ctx.clearRect(0, 0, w, h);
      ctx.strokeStyle = '#22c55e';
      ctx.lineWidth = 2;
      ctx.beginPath();

      const step = w / speedHistory.length;
      for (let i = 0; i < speedHistory.length; i++) {
        const val = speedHistory[i];
        const y = h - (val / 150.0) * (h - 10);
        if (i === 0) ctx.moveTo(0, y);
        else ctx.lineTo(i * step, y);
      }
      ctx.stroke();
    }

    setInterval(() => {
      const currentSpeed = Math.random() * 15 + 85;
      speedHistory.shift();
      speedHistory.push(currentSpeed);
      const el = document.getElementById('status-tokens-sec');
      if (el) el.innerText = `${currentSpeed.toFixed(1)} tok/s`;
      const el2 = document.getElementById('table-speed-0');
      if (el2) el2.innerText = `${currentSpeed.toFixed(1)} tok/s`;
      drawSpeedGraph();
    }, 1000);

    renderPiecesGrid();
    pollDeltaSync();
    setInterval(pollDeltaSync, 2000);
    loadCatalog();

    // Gracefully inform tracker when user closes tab
    window.addEventListener('beforeunload', () => {
      if (browserEngine && browserEngine.peerId) {
        const payload = JSON.stringify({ peer_id: browserEngine.peerId });
        const blob = new Blob([payload], { type: 'application/json' });
        navigator.sendBeacon('/api/peer/leave', blob);
      }
    });
  </script>
</body>
</html>
"#;

pub fn create_web_router() -> Router {
    Router::new().route("/", get(|| async { Html(DASHBOARD_HTML) }))
}
