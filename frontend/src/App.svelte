<script>
  import { onMount, onDestroy } from 'svelte';
  import './app.css';

  // State: cart items list
  let cartItems = [];
  let lastScannedCode = '';
  let notFoundMessage = '';
  let showManualModal = false;
  let manualInput = '';

  // Settings
  let serverHost = '';
  let sucursal = '01';

  // Inactivity timer state (6 seconds after last scan, auto-clears cart for next customer)
  let timerMax = 6;
  let timerCurrent = 6;
  let timerInterval = null;

  // Keydown buffer for USB/Bluetooth HID Barcode Scanners and Keyboard
  let inputBuffer = '';
  let lastKeyTime = 0;

  $: cartTotal = cartItems.reduce((acc, item) => acc + item.precio * item.qty, 0);

  onMount(() => {
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
    if (showManualModal) return;

    const now = Date.now();

    if (now - lastKeyTime > 2000) {
      inputBuffer = '';
    }
    lastKeyTime = now;

    if (e.key === 'Enter') {
      e.preventDefault();
      if (inputBuffer.trim().length > 0) {
        const codeToSearch = inputBuffer.trim();
        inputBuffer = '';
        fetchAndAccumulate(codeToSearch);
      }
    } else if (e.key.length === 1) {
      inputBuffer += e.key;
    }
  }

  async function fetchAndAccumulate(code) {
    clearAutoTimer();
    lastScannedCode = code;
    notFoundMessage = '';
    showManualModal = false;

    try {
      const baseUrl = serverHost || window.location.origin;
      const url = `${baseUrl}/api/producto?codigo=${encodeURIComponent(code)}&sucursal=${encodeURIComponent(sucursal)}`;
      
      const res = await fetch(url);
      if (res.ok) {
        const data = await res.json();
        if (data && data.encontrado) {
          // Check if item already exists in cart -> increment quantity
          const existingIndex = cartItems.findIndex(i => i.codigo === data.codigo);
          if (existingIndex >= 0) {
            cartItems[existingIndex].qty += 1;
            cartItems = [...cartItems];
          } else {
            cartItems = [{
              codigo: data.codigo,
              nombre: data.nombre,
              precio: data.precio,
              existencia: data.existencia,
              unidad: data.unidad || 'UND',
              qty: 1
            }, ...cartItems];
          }
          startAutoTimer(6);
          return;
        }
      }
      
      notFoundMessage = `Código ${code} no registrado`;
      startAutoTimer(6);
    } catch (err) {
      console.error('Fetch error:', err);
      notFoundMessage = 'Error de conexión con el servidor';
      startAutoTimer(6);
    }
  }

  function handleManualSubmit() {
    if (manualInput.trim()) {
      fetchAndAccumulate(manualInput.trim());
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
        clearCart();
      }
    }, intervalMs);
  }

  function clearAutoTimer() {
    if (timerInterval) {
      clearInterval(timerInterval);
      timerInterval = null;
    }
  }

  function clearCart() {
    clearAutoTimer();
    cartItems = [];
    lastScannedCode = '';
    notFoundMessage = '';
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
  <!-- Top Bar: Header & Total Counter -->
  <div style="width: 100%; max-width: 800px; display: flex; align-items: center; justify-content: space-between; margin-bottom: 1.5rem; padding: 0 0.5rem;">
    <div style="text-align: left;">
      <h1 style="font-size: 1.8rem; font-weight: 800; color: var(--text-main); letter-spacing: -0.5px;">
        VERIFICADOR DE PRECIOS
      </h1>
      <p style="font-size: 0.95rem; color: var(--text-muted);">
        Escanee sus productos consecutivamente
      </p>
    </div>

    {#if cartItems.length > 0}
      <button 
        on:click={clearCart}
        style="background: rgba(220,38,38,0.1); border: 1px solid rgba(220,38,38,0.2); color: var(--accent-red); padding: 0.5rem 1rem; border-radius: 100px; font-size: 0.85rem; font-weight: 600; cursor: pointer;"
      >
        🧹 Limpiar
      </button>
    {/if}
  </div>

  <!-- Main Content Area -->
  <div class="card" style="max-width: 800px; min-height: 420px; display: flex; flex-direction: column; justify-content: space-between;">
    
    {#if cartItems.length === 0 && !notFoundMessage}
      <!-- Standby State -->
      <div style="margin: auto 0; padding: 2rem 0;">
        <div class="scanner-box" on:click={() => (showManualModal = true)} role="button" tabindex="0">
          <svg width="44" height="44" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
            <path stroke-linecap="round" stroke-linejoin="round" d="M3.75 4.875c0-.621.504-1.125 1.125-1.125h4.5c.621 0 1.125.504 1.125 1.125v4.5c0 .621-.504 1.125-1.125 1.125h-4.5A1.125 1.125 0 013.75 9.375v-4.5zM3.75 14.625c0-.621.504-1.125 1.125-1.125h4.5c.621 0 1.125.504 1.125 1.125v4.5c0 .621-.504 1.125-1.125 1.125h-4.5a1.125 1.125 0 01-1.125-1.125v-4.5zM13.5 4.875c0-.621.504-1.125 1.125-1.125h4.5c.621 0 1.125.504 1.125 1.125v4.5c0 .621-.504 1.125-1.125 1.125h-4.5A1.125 1.125 0 0113.5 9.375v-4.5z" />
            <path stroke-linecap="round" stroke-linejoin="round" d="M6.75 6.75h.008v.008H6.75V6.75zM6.75 16.5h.008v.008H6.75V16.5zM16.5 6.75h.008v.008H16.5V6.75zM13.5 13.5h3v3h-3zM16.5 16.5h3v3h-3zM13.5 19.5h3v.008h-3z" />
          </svg>
        </div>
        <h2 style="font-size: 1.8rem; font-weight: 700; color: var(--text-main); margin-bottom: 0.5rem;">
          Pase el código por el lector
        </h2>
        <p style="font-size: 1.1rem; color: var(--text-muted);">
          Los precios se irán sumando automáticamente
        </p>
      </div>

    {:else}
      <!-- Accumulated Items List -->
      <div style="flex: 1; overflow-y: auto; max-height: 280px; margin-bottom: 1rem; text-align: left;">
        
        {#if notFoundMessage}
          <div style="background: rgba(220,38,38,0.08); border: 1px solid rgba(220,38,38,0.2); color: var(--accent-red); padding: 0.8rem 1.2rem; border-radius: 14px; margin-bottom: 1rem; font-weight: 600;">
            ⚠️ {notFoundMessage}
          </div>
        {/if}

        <table style="width: 100%; border-collapse: collapse;">
          <thead>
            <tr style="border-bottom: 1px solid #e2e8f0; font-size: 0.85rem; color: var(--text-muted); text-transform: uppercase;">
              <th style="padding: 0.5rem; text-align: left;">Producto</th>
              <th style="padding: 0.5rem; text-align: center;">Cant.</th>
              <th style="padding: 0.5rem; text-align: right;">Precio</th>
              <th style="padding: 0.5rem; text-align: right;">Subtotal</th>
            </tr>
          </thead>
          <tbody>
            {#each cartItems as item}
              <tr style="border-bottom: 1px solid #f1f5f9; font-size: 1.05rem;">
                <td style="padding: 0.8rem 0.5rem; font-weight: 600;">
                  {item.nombre}
                  <div style="font-size: 0.8rem; color: var(--text-muted); font-family: monospace;">#{item.codigo}</div>
                </td>
                <td style="padding: 0.8rem 0.5rem; text-align: center; font-weight: 700; color: var(--accent-blue);">
                  x{item.qty}
                </td>
                <td style="padding: 0.8rem 0.5rem; text-align: right; color: var(--text-muted);">
                  {formatCurrency(item.precio)}
                </td>
                <td style="padding: 0.8rem 0.5rem; text-align: right; font-weight: 700; color: var(--text-main);">
                  {formatCurrency(item.precio * item.qty)}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}

    <!-- Total Banner at Bottom -->
    <div style="background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 20px; padding: 1.2rem 1.5rem; display: flex; align-items: center; justify-content: space-between; margin-top: 1rem;">
      <div style="text-align: left;">
        <span style="font-size: 0.85rem; color: var(--text-muted); text-transform: uppercase; font-weight: 600; letter-spacing: 1px;">TOTAL ACUMULADO</span>
        <div style="font-size: 2.8rem; font-weight: 900; color: var(--accent-green); line-height: 1;">
          {formatCurrency(cartTotal)}
        </div>
      </div>

      <button 
        on:click={() => (showManualModal = true)}
        style="background: #ffffff; border: 1px solid #cbd5e1; color: var(--text-main); padding: 0.75rem 1.2rem; border-radius: 100px; font-size: 0.95rem; font-weight: 600; cursor: pointer; display: flex; align-items: center; gap: 6px; box-shadow: 0 2px 6px rgba(0,0,0,0.04);"
      >
        <span>🔍</span>
        <span>Código manual</span>
      </button>
    </div>

    <!-- Timer progress bar that clears after 6 seconds of inactivity -->
    {#if cartItems.length > 0 || notFoundMessage}
      <div class="timer-bar" style="width: {(timerCurrent / timerMax) * 100}%"></div>
    {/if}
  </div>

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
            <button type="submit" class="minimal-btn" style="flex: 1;">Agregar / Sumar</button>
            <button type="button" class="minimal-btn" on:click={() => (showManualModal = false)} style="flex: 1; background: #e2e8f0; color: var(--text-main);">Cancelar</button>
          </div>
        </form>
      </div>
    </div>
  {/if}
</div>
