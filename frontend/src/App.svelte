<script>
  import { onMount, onDestroy } from 'svelte';
  import './app.css';

  // Application States: 'IDLE' | 'LOADING' | 'FOUND' | 'NOT_FOUND' | 'SETTINGS'
  let state = 'IDLE';

  let product = null;
  let scannedCode = '';
  let manualInput = '';
  let showManualModal = false;

  // Config settings
  let serverHost = '';
  let sucursal = '01';

  // Timer state
  let timerMax = 5;
  let timerCurrent = 5;
  let timerInterval = null;

  // Keydown buffer for USB/Bluetooth HID Barcode Scanners and Keyboard
  let inputBuffer = '';
  let lastKeyTime = 0;

  onMount(() => {
    serverHost = localStorage.getItem('vgs_server_host') || window.location.origin;
    sucursal = localStorage.getItem('vgs_sucursal') || '01';

    // Global listener listening to ALL keydown events across the entire window
    window.addEventListener('keydown', handleGlobalKeydown);
  });

  onDestroy(() => {
    if (typeof window !== 'undefined') {
      window.removeEventListener('keydown', handleGlobalKeydown);
    }
    clearAutoTimer();
  });

  function handleGlobalKeydown(e) {
    // Do not capture barcode scanner if editing inside settings modal or manual search input modal
    if (state === 'SETTINGS' || showManualModal) return;

    const now = Date.now();

    // Reset buffer if more than 2 seconds (2000ms) pass without typing
    if (now - lastKeyTime > 2000) {
      inputBuffer = '';
    }
    lastKeyTime = now;

    if (e.key === 'Enter') {
      e.preventDefault();
      if (inputBuffer.trim().length > 0) {
        const codeToSearch = inputBuffer.trim();
        inputBuffer = '';
        fetchProduct(codeToSearch);
      }
    } else if (e.key.length === 1) {
      // Append alphanumeric barcode characters
      inputBuffer += e.key;
    }
  }

  async function fetchProduct(code) {
    clearAutoTimer();
    scannedCode = code;
    showManualModal = false;
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
      state = 'NOT_FOUND';
      startAutoTimer(3);
    }
  }

  function handleManualSubmit() {
    if (manualInput.trim()) {
      fetchProduct(manualInput.trim());
      manualInput = '';
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
      maximumFractionDigits: 0
    }).format(amount);
  }
</script>

<div class="kiosk-container">
  <!-- Gear Icon for Settings (Top Right) -->
  <button class="settings-btn" on:click={() => (state = state === 'SETTINGS' ? 'IDLE' : 'SETTINGS')} title="Configuración">
    ⚙️
  </button>

  {#if state === 'IDLE'}
    <div style="display: flex; flex-direction: column; align-items: center; max-width: 600px; width: 100%;">
      <div class="scanner-box" on:click={() => (showManualModal = true)} role="button" tabindex="0" title="Búsqueda manual">
        <svg width="44" height="44" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
          <path stroke-linecap="round" stroke-linejoin="round" d="M3.75 4.875c0-.621.504-1.125 1.125-1.125h4.5c.621 0 1.125.504 1.125 1.125v4.5c0 .621-.504 1.125-1.125 1.125h-4.5A1.125 1.125 0 013.75 9.375v-4.5zM3.75 14.625c0-.621.504-1.125 1.125-1.125h4.5c.621 0 1.125.504 1.125 1.125v4.5c0 .621-.504 1.125-1.125 1.125h-4.5a1.125 1.125 0 01-1.125-1.125v-4.5zM13.5 4.875c0-.621.504-1.125 1.125-1.125h4.5c.621 0 1.125.504 1.125 1.125v4.5c0 .621-.504 1.125-1.125 1.125h-4.5A1.125 1.125 0 0113.5 9.375v-4.5z" />
          <path stroke-linecap="round" stroke-linejoin="round" d="M6.75 6.75h.008v.008H6.75V6.75zM6.75 16.5h.008v.008H6.75V16.5zM16.5 6.75h.008v.008H16.5V6.75zM13.5 13.5h3v3h-3zM16.5 16.5h3v3h-3zM13.5 19.5h3v.008h-3z" />
        </svg>
      </div>

      <h1 style="font-size: 2.8rem; font-weight: 800; tracking: -1px; margin-bottom: 0.8rem;">
        Consultar Precio
      </h1>
      <p style="font-size: 1.3rem; color: var(--text-muted); margin-bottom: 2.5rem;">
        Escanee su producto con el lector de código de barras
      </p>

      <!-- Minimalist Manual Entry Link (Fallback if no physical scanner) -->
      <div style="width: 100%; max-width: 360px;">
        <button 
          on:click={() => (showManualModal = true)}
          style="width: 100%; background: #ffffff; border: 1px solid #cbd5e1; color: var(--text-muted); padding: 0.75rem 1.2rem; border-radius: 100px; font-size: 0.95rem; font-weight: 500; cursor: pointer; display: flex; align-items: center; justify-content: center; gap: 8px; box-shadow: 0 2px 8px rgba(0,0,0,0.03); transition: all 0.2s;"
        >
          <span>⌨️</span>
          <span>Búsqueda manual por teclado</span>
        </button>
      </div>
    </div>

  {:else if state === 'LOADING'}
    <div class="card">
      <div class="product-code">BUSCANDO EN BASE DE DATOS...</div>
      <h2 class="product-title" style="margin-top: 1rem; color: var(--accent-blue);">
        {scannedCode}
      </h2>
    </div>

  {:else if state === 'FOUND' && product}
    <div class="card">
      <div class="product-code">CÓDIGO #{product.codigo}</div>
      <h2 class="product-title">{product.nombre}</h2>
      
      <div class="price-tag">{formatCurrency(product.precio)}</div>
      
      {#if product.existencia !== undefined}
        <div class="stock-badge">
          <span>Disponibles:</span>
          <strong style="color: var(--text-main);">{product.existencia} {product.unidad || 'UND'}</strong>
        </div>
      {/if}

      <div class="timer-bar" style="width: {(timerCurrent / timerMax) * 100}%"></div>
    </div>

  {:else if state === 'NOT_FOUND'}
    <div class="card not-found-card">
      <div class="product-code" style="color: var(--accent-red);">CÓDIGO ESCANEADO: {scannedCode}</div>
      <h2 class="not-found-title" style="margin-top: 1rem;">Producto no registrado</h2>
      <p style="font-size: 1.1rem; color: var(--text-muted); margin-top: 0.5rem;">
        Consulte con un asesor de servicio en tienda
      </p>
      
      <div class="timer-bar" style="background: var(--accent-red); width: {(timerCurrent / timerMax) * 100}%"></div>
    </div>

  {:else if state === 'SETTINGS'}
    <div class="card" style="max-width: 480px; text-align: left;">
      <h2 style="font-size: 1.6rem; font-weight: 700; margin-bottom: 1.5rem;">⚙️ Configuración</h2>
      
      <div style="margin-bottom: 1.2rem;">
        <label for="serverHostInput" style="display: block; font-size: 0.9rem; color: var(--text-muted); margin-bottom: 0.5rem;">IP / URL Servidor Local:</label>
        <input 
          id="serverHostInput"
          type="text" 
          class="minimal-input"
          bind:value={serverHost} 
          placeholder="http://192.168.1.50:8080" 
        />
      </div>

      <div style="margin-bottom: 2rem;">
        <label for="sucursalInput" style="display: block; font-size: 0.9rem; color: var(--text-muted); margin-bottom: 0.5rem;">Código de Sucursal (SUCCOD):</label>
        <input 
          id="sucursalInput"
          type="text" 
          class="minimal-input"
          bind:value={sucursal} 
          placeholder="01" 
        />
      </div>

      <div style="display: flex; gap: 0.8rem;">
        <button class="minimal-btn" on:click={saveSettings} style="flex: 1; background: var(--accent-green);">
          Guardar
        </button>
        <button class="minimal-btn" on:click={() => (state = 'IDLE')} style="flex: 1; background: #e2e8f0; color: var(--text-main);">
          Cerrar
        </button>
      </div>
    </div>
  {/if}

  <!-- Manual Entry Modal -->
  {#if showManualModal}
    <div style="position: fixed; inset: 0; background: rgba(15, 23, 42, 0.4); backdrop-filter: blur(8px); display: flex; align-items: center; justify-content: center; z-index: 100; padding: 1.5rem;">
      <div class="card" style="max-width: 440px; text-align: left; background: #ffffff;">
        <h3 style="font-size: 1.4rem; font-weight: 700; margin-bottom: 0.5rem;">Búsqueda Manual</h3>
        <p style="font-size: 0.95rem; color: var(--text-muted); margin-bottom: 1.2rem;">Digite el código del producto:</p>
        
        <form on:submit|preventDefault={handleManualSubmit}>
          <input 
            type="text" 
            class="minimal-input" 
            bind:value={manualInput} 
            placeholder="Ingrese código de producto" 
            style="margin-bottom: 1.5rem;"
          />
          <div style="display: flex; gap: 0.8rem;">
            <button type="submit" class="minimal-btn" style="flex: 1;">Consultar</button>
            <button type="button" class="minimal-btn" on:click={() => (showManualModal = false)} style="flex: 1; background: #e2e8f0; color: var(--text-main);">Cancelar</button>
          </div>
        </form>
      </div>
    </div>
  {/if}
</div>
