<script>
  import { onMount, onDestroy } from 'svelte';
  import '../app.css';

  // Application States: 'IDLE' | 'LOADING' | 'FOUND' | 'NOT_FOUND' | 'SETTINGS'
  let state = 'IDLE';

  let product = null;
  let scannedCode = '';
  let errorMessage = '';

  // Config settings
  let serverHost = '';
  let sucursal = '01';

  // Timer state
  let timerMax = 5;
  let timerCurrent = 5;
  let timerInterval = null;

  // Keydown buffer for HID barcode scanner
  let inputBuffer = '';
  let lastKeyTime = 0;

  onMount(() => {
    // Load config from localStorage if set
    serverHost = localStorage.getItem('vgs_server_host') || window.location.origin;
    sucursal = localStorage.getItem('vgs_sucursal') || '01';

    window.addEventListener('keydown', handleGlobalKeydown);
  });

  onDestroy(() => {
    if (typeof window !== 'undefined') {
      window.removeEventListener('keydown', handleGlobalKeydown);
    }
    clearAutoTimer();
  });

  function handleGlobalKeydown(e) {
    // Ignore input if editing in settings modal inputs
    if (state === 'SETTINGS' && (e.target.tagName === 'INPUT' || e.target.tagName === 'SELECT')) {
      return;
    }

    const now = Date.now();

    // Reset buffer if delay between keystrokes is too long (> 200ms)
    if (now - lastKeyTime > 200) {
      inputBuffer = '';
    }
    lastKeyTime = now;

    if (e.key === 'Enter') {
      if (inputBuffer.trim().length > 0) {
        const codeToSearch = inputBuffer.trim();
        inputBuffer = '';
        fetchProduct(codeToSearch);
      }
    } else if (e.key.length === 1) {
      inputBuffer += e.key;
    }
  }

  async function fetchProduct(code) {
    clearAutoTimer();
    scannedCode = code;
    state = 'LOADING';

    try {
      const baseUrl = serverHost || window.location.origin;
      const url = `${baseUrl}/api/producto?codigo=${encodeURIComponent(code)}&sucursal=${encodeURIComponent(sucursal)}`;
      
      const res = await fetch(url);
      if (res.ok) {
        const data = await res.json();
        if (data && data.encontrado) {
          product = data;
          state = 'FOUND';
          startAutoTimer(5);
          return;
        }
      }
      
      state = 'NOT_FOUND';
      startAutoTimer(3);
    } catch (err) {
      console.error('Fetch error:', err);
      errorMessage = 'Error de conexión con el servidor local';
      state = 'NOT_FOUND';
      startAutoTimer(3);
    }
  }

  function startAutoTimer(seconds) {
    clearAutoTimer();
    timerMax = seconds;
    timerCurrent = seconds;

    const intervalMs = 100;
    const decrement = intervalMs / 1000;

    timerInterval = setInterval(() => {
      timerCurrent -= decrement;
      if (timerCurrent <= 0) {
        clearAutoTimer();
        state = 'IDLE';
        product = null;
      }
    }, intervalMs);
  }

  function clearAutoTimer() {
    if (timerInterval) {
      clearInterval(timerInterval);
      timerInterval = null;
    }
  }

  function saveSettings() {
    localStorage.setItem('vgs_server_host', serverHost);
    localStorage.setItem('vgs_sucursal', sucursal);
    state = 'IDLE';
  }

  function formatCurrency(amount) {
    if (amount === undefined || amount === null) return '$ 0';
    return new Intl.NumberFormat('es-CO', {
      style: 'currency',
      currency: 'COP',
      minimumFractionDigits: 0,
      maximumFractionDigits: 2
    }).format(amount);
  }
</script>

<div class="kiosk-container">
  <!-- Gear Icon for Settings (Top Right) -->
  <button class="settings-btn" on:click={() => (state = state === 'SETTINGS' ? 'IDLE' : 'SETTINGS')} title="Configuración">
    ⚙️
  </button>

  {#if state === 'IDLE'}
    <div style="cursor: pointer;" on:click={() => (state = 'SETTINGS')}>
      <svg class="scanner-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path stroke-linecap="round" stroke-linejoin="round" d="M3.75 4.875c0-.621.504-1.125 1.125-1.125h4.5c.621 0 1.125.504 1.125 1.125v4.5c0 .621-.504 1.125-1.125 1.125h-4.5A1.125 1.125 0 013.75 9.375v-4.5zM3.75 14.625c0-.621.504-1.125 1.125-1.125h4.5c.621 0 1.125.504 1.125 1.125v4.5c0 .621-.504 1.125-1.125 1.125h-4.5a1.125 1.125 0 01-1.125-1.125v-4.5zM13.5 4.875c0-.621.504-1.125 1.125-1.125h4.5c.621 0 1.125.504 1.125 1.125v4.5c0 .621-.504 1.125-1.125 1.125h-4.5A1.125 1.125 0 0113.5 9.375v-4.5z" />
        <path stroke-linecap="round" stroke-linejoin="round" d="M6.75 6.75h.008v.008H6.75V6.75zM6.75 16.5h.008v.008H6.75V16.5zM16.5 6.75h.008v.008H16.5V6.75zM13.5 13.5h3v3h-3zM16.5 16.5h3v3h-3zM13.5 19.5h3v.008h-3z" />
      </svg>
    </div>
    <h1 style="font-size: 3rem; font-weight: 800; margin-bottom: 1rem;">VERIFICADOR DE PRECIOS</h1>
    <p style="font-size: 1.8rem; color: var(--text-muted);">Pase el código de barras por el lector</p>

  {:else if state === 'LOADING'}
    <div class="card">
      <h2 class="product-title">Buscando producto...</h2>
      <p style="font-size: 1.4rem; color: var(--accent-blue);">Código: {scannedCode}</p>
    </div>

  {:else if state === 'FOUND' && product}
    <div class="card">
      <div class="product-code">CÓDIGO: {product.codigo}</div>
      <h2 class="product-title">{product.nombre}</h2>
      
      <div class="price-tag">{formatCurrency(product.precio)}</div>
      
      {#if product.existencia !== undefined}
        <div class="stock-info">
          Stock disponible: {product.existencia} {product.unidad || 'UND'}
        </div>
      {/if}

      <div class="timer-bar" style="width: {(timerCurrent / timerMax) * 100}%"></div>
    </div>

  {:else if state === 'NOT_FOUND'}
    <div class="card not-found-card">
      <div style="font-size: 4rem; margin-bottom: 1rem;">⚠️</div>
      <h2 class="not-found-title">PRODUCTO NO REGISTRADO</h2>
      <p style="font-size: 1.4rem; color: var(--text-muted); margin-bottom: 1rem;">
        Código escaneado: <strong style="color: var(--text-main); font-family: monospace;">{scannedCode}</strong>
      </p>
      <p style="font-size: 1.1rem; color: var(--accent-gold);">Por favor solicite asistencia con un asesor</p>
      
      <div class="timer-bar" style="background: var(--accent-red); width: {(timerCurrent / timerMax) * 100}%"></div>
    </div>

  {:else if state === 'SETTINGS'}
    <div class="card" style="max-width: 500px;">
      <h2 style="font-size: 1.8rem; margin-bottom: 1.5rem;">⚙️ Configuración del Servidor</h2>
      
      <div style="text-align: left; margin-bottom: 1.2rem;">
        <label style="display: block; font-weight: 600; margin-bottom: 0.4rem;">Servidor Local (URL):</label>
        <input 
          type="text" 
          bind:value={serverHost} 
          placeholder="http://192.168.1.50:8080" 
          style="width: 100%; padding: 0.8rem; border-radius: 8px; border: 1px solid rgba(255,255,255,0.2); background: rgba(0,0,0,0.3); color: white; font-size: 1rem;"
        />
      </div>

      <div style="text-align: left; margin-bottom: 1.5rem;">
        <label style="display: block; font-weight: 600; margin-bottom: 0.4rem;">Código de Sucursal (SUCCOD):</label>
        <input 
          type="text" 
          bind:value={sucursal} 
          placeholder="01" 
          style="width: 100%; padding: 0.8rem; border-radius: 8px; border: 1px solid rgba(255,255,255,0.2); background: rgba(0,0,0,0.3); color: white; font-size: 1rem;"
        />
      </div>

      <div style="display: flex; gap: 1rem;">
        <button 
          on:click={saveSettings} 
          style="flex: 1; padding: 0.8rem; background: var(--accent-green); color: white; font-weight: bold; border: none; border-radius: 8px; cursor: pointer; font-size: 1rem;"
        >
          Guardar
        </button>
        <button 
          on:click={() => (state = 'IDLE')} 
          style="flex: 1; padding: 0.8rem; background: rgba(255,255,255,0.1); color: white; border: none; border-radius: 8px; cursor: pointer; font-size: 1rem;"
        >
          Cancelar
        </button>
      </div>
    </div>
  {/if}
</div>
