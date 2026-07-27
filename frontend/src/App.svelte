<script>
  import { onMount, onDestroy } from 'svelte';
  import './app.css';

  // State: cart items list
  let cartItems = [];
  let lastScannedCode = '';
  let notFoundMessage = '';
  let isOfflineMode = false;
  
  // Security & Modals
  let showManualModal = false;
  let showPinModal = false;
  let showSettingsModal = false;
  let manualInput = '';
  let adminPinInput = '';
  let pinError = '';
  let logoTapCount = 0;
  let logoTapTimer = null;

  // Admin PIN (default 7612)
  const ADMIN_PIN = '7612';

  // Settings
  let serverHost = '';
  let sucursal = '01';

  // Inactivity timer state (6 seconds after last scan, auto-clears cart)
  let timerMax = 6;
  let timerCurrent = 6;
  let timerInterval = null;

  // Keydown buffer for USB/Bluetooth HID Barcode Scanners and Keyboard
  let inputBuffer = '';
  let lastKeyTime = 0;

  // IndexedDB reference for offline resilience
  let db = null;

  $: cartTotal = cartItems.reduce((acc, item) => acc + item.precio * item.qty, 0);

  onMount(() => {
    serverHost = localStorage.getItem('vgs_server_host') || window.location.origin;
    sucursal = localStorage.getItem('vgs_sucursal') || '01';

    window.addEventListener('keydown', handleGlobalKeydown);
    window.addEventListener('online', handleOnlineStatus);
    window.addEventListener('offline', handleOfflineStatus);

    if (typeof navigator !== 'undefined' && !navigator.onLine) {
      isOfflineMode = true;
    }

    // Initialize IndexedDB and start background sync
    initIndexedDB().then(() => {
      syncOfflineData();
    });
  });

  onDestroy(() => {
    if (typeof window !== 'undefined') {
      window.removeEventListener('keydown', handleGlobalKeydown);
      window.removeEventListener('online', handleOnlineStatus);
      window.removeEventListener('offline', handleOfflineStatus);
    }
    clearAutoTimer();
  });

  function handleOnlineStatus() {
    isOfflineMode = false;
    syncOfflineData();
  }

  function handleOfflineStatus() {
    isOfflineMode = true;
  }

  // IndexedDB Native Initializer
  function initIndexedDB() {
    return new Promise((resolve) => {
      if (typeof window === 'undefined' || !('indexedDB' in window)) {
        resolve();
        return;
      }

      const request = window.indexedDB.open('vgs_offline_db', 1);

      request.onupgradeneeded = (e) => {
        const database = e.target.result;
        if (!database.objectStoreNames.contains('productos')) {
          database.createObjectStore('productos', { keyPath: 'c' });
        }
      };

      request.onsuccess = (e) => {
        db = e.target.result;
        resolve();
      };

      request.onerror = () => {
        resolve();
      };
    });
  }

  // Background bulk sync from Rust backend into IndexedDB
  async function syncOfflineData() {
    if (typeof navigator !== 'undefined' && !navigator.onLine) return;

    try {
      const baseUrl = serverHost || window.location.origin;
      const res = await fetch(`${baseUrl}/api/productos/sync?sucursal=${encodeURIComponent(sucursal)}`);
      if (res.ok) {
        const items = await res.json();
        if (db && Array.isArray(items) && items.length > 0) {
          const tx = db.transaction('productos', 'readwrite');
          const store = tx.objectStore('productos');
          items.forEach(item => store.put(item));
        }
        isOfflineMode = false;
      }
    } catch (err) {
      console.warn('Sync background offline:', err);
    }
  }

  // Search local IndexedDB fallback (Instant <1ms lookup)
  function searchOfflineDB(code) {
    return new Promise((resolve) => {
      if (!db) {
        resolve(null);
        return;
      }
      const tx = db.transaction('productos', 'readonly');
      const store = tx.objectStore('productos');
      const request = store.get(code);

      request.onsuccess = () => {
        if (request.result) {
          const res = request.result;
          resolve({
            codigo: res.c,
            nombre: res.n,
            precio: res.p,
            existencia: res.e,
            unidad: res.u || 'UND',
            encontrado: true
          });
        } else {
          resolve(null);
        }
      };

      request.onerror = () => resolve(null);
    });
  }

  function handleGlobalKeydown(e) {
    if (showManualModal || showPinModal || showSettingsModal) return;

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

    let productData = null;

    // Si ya detectamos offline o navigator.onLine es false -> CONSULTA INSTANTÁNEA EN INDEXEDDB (0ms delay)
    if (isOfflineMode || (typeof navigator !== 'undefined' && !navigator.onLine)) {
      productData = await searchOfflineDB(code);
    } else {
      // Si estamos online -> consultar red con timeout ultra rápido de 800ms
      try {
        const baseUrl = serverHost || window.location.origin;
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), 800); // 800ms max

        const res = await fetch(`${baseUrl}/api/producto?codigo=${encodeURIComponent(code)}&sucursal=${encodeURIComponent(sucursal)}`, {
          signal: controller.signal
        });
        clearTimeout(timeoutId);

        if (res.ok) {
          const data = await res.json();
          if (data && data.encontrado) {
            productData = data;
            isOfflineMode = false;
          }
        }
      } catch (err) {
        console.warn('Fallo de red o timeout, buscando instantáneamente en IndexedDB...', err);
        isOfflineMode = true;
        productData = await searchOfflineDB(code);
      }
    }

    if (productData) {
      const existingIndex = cartItems.findIndex(i => i.codigo === productData.codigo);
      if (existingIndex >= 0) {
        cartItems[existingIndex].qty += 1;
        cartItems = [...cartItems];
      } else {
        cartItems = [{
          codigo: productData.codigo,
          nombre: productData.nombre,
          precio: productData.precio,
          existencia: productData.existencia,
          unidad: productData.unidad || 'UND',
          qty: 1
        }, ...cartItems];
      }
      startAutoTimer(6);
    } else {
      notFoundMessage = `Código ${code} no registrado`;
      startAutoTimer(6);
    }
  }

  function handleManualSubmit() {
    if (manualInput.trim()) {
      fetchAndAccumulate(manualInput.trim());
      manualInput = '';
    }
  }

  // Secret 3-tap gesture on Title to prompt PIN for Settings
  function handleSecretTitleTap() {
    logoTapCount += 1;
    if (logoTapTimer) clearTimeout(logoTapTimer);

    if (logoTapCount >= 3) {
      logoTapCount = 0;
      adminPinInput = '';
      pinError = '';
      showPinModal = true;
    } else {
      logoTapTimer = setTimeout(() => {
        logoTapCount = 0;
      }, 1000);
    }
  }

  function verifyAdminPin() {
    if (adminPinInput === ADMIN_PIN) {
      showPinModal = false;
      showSettingsModal = true;
      pinError = '';
    } else {
      pinError = 'Clave incorrecta (PIN Administrador)';
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

  function saveSettings() {
    localStorage.setItem('vgs_server_host', serverHost);
    localStorage.setItem('vgs_sucursal', sucursal);
    showSettingsModal = false;
    syncOfflineData();
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
  <!-- Top Bar: Header Title & Status -->
  <div style="width: 100%; max-width: 800px; display: flex; align-items: center; justify-content: space-between; margin-bottom: 1.5rem; padding: 0 0.5rem;">
    <!-- Secret gesture trigger on title: 3 taps opens Admin PIN modal -->
    <div style="text-align: left; cursor: default;" on:click={handleSecretTitleTap} role="button" tabindex="0">
      <div style="display: flex; align-items: center; gap: 10px;">
        <h1 style="font-size: 1.8rem; font-weight: 800; color: var(--text-main); letter-spacing: -0.5px;">
          VERIFICADOR DE PRECIOS
        </h1>
        {#if isOfflineMode}
          <span style="background: rgba(217,119,6,0.12); color: #d97706; border: 1px solid rgba(217,119,6,0.3); padding: 0.25rem 0.75rem; border-radius: 100px; font-size: 0.75rem; font-weight: 800; letter-spacing: 0.5px;">OFFLINE</span>
        {/if}
      </div>
      <p style="font-size: 0.95rem; color: var(--text-muted);">
        Pase sus productos por el lector de barras
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
  <div class="card" style="max-width: 800px; min-height: 400px; display: flex; flex-direction: column; justify-content: space-between;">
    
    {#if cartItems.length === 0 && !notFoundMessage}
      <!-- Standby Clean State -->
      <div style="margin: auto 0; padding: 3rem 1rem; display: flex; flex-direction: column; align-items: center; justify-content: center;">
        <div class="scanner-box" on:click={() => (showManualModal = true)} role="button" tabindex="0" style="margin-bottom: 1.5rem;">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
            <path stroke-linecap="round" stroke-linejoin="round" d="M3.75 4.875c0-.621.504-1.125 1.125-1.125h4.5c.621 0 1.125.504 1.125 1.125v4.5c0 .621-.504 1.125-1.125 1.125h-4.5A1.125 1.125 0 013.75 9.375v-4.5zM3.75 14.625c0-.621.504-1.125 1.125-1.125h4.5c.621 0 1.125.504 1.125 1.125v4.5c0 .621-.504 1.125-1.125 1.125h-4.5a1.125 1.125 0 01-1.125-1.125v-4.5zM13.5 4.875c0-.621.504-1.125 1.125-1.125h4.5c.621 0 1.125.504 1.125 1.125v4.5c0 .621-.504 1.125-1.125 1.125h-4.5A1.125 1.125 0 0113.5 9.375v-4.5z" />
            <path stroke-linecap="round" stroke-linejoin="round" d="M6.75 6.75h.008v.008H6.75V6.75zM6.75 16.5h.008v.008H6.75V16.5zM16.5 6.75h.008v.008H16.5V6.75zM13.5 13.5h3v3h-3zM16.5 16.5h3v3h-3zM13.5 19.5h3v.008h-3z" />
          </svg>
        </div>
        <h2 style="font-size: 1.9rem; font-weight: 800; color: var(--text-main); margin-bottom: 0.6rem;">
          Pase el código por el lector de barras
        </h2>
        <p style="font-size: 1.05rem; color: var(--text-muted); margin-bottom: 1.8rem;">
          Los productos escaneados aparecerán en pantalla
        </p>

        <button 
          on:click={() => (showManualModal = true)}
          style="background: #ffffff; border: 1.5px solid #cbd5e1; color: var(--text-main); padding: 0.8rem 1.4rem; border-radius: 100px; font-size: 0.95rem; font-weight: 700; cursor: pointer; display: flex; align-items: center; gap: 8px; box-shadow: 0 2px 8px rgba(0,0,0,0.04);"
        >
          <span>🔍</span>
          <span>Ingresar código manual</span>
        </button>
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

      <!-- Total Banner at Bottom (ONLY shown when products exist) -->
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
    {/if}

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

  <!-- Admin PIN Security Modal -->
  {#if showPinModal}
    <div style="position: fixed; inset: 0; background: rgba(15, 23, 42, 0.6); backdrop-filter: blur(8px); display: flex; align-items: center; justify-content: center; z-index: 120; padding: 1.5rem;">
      <div class="card" style="max-width: 400px; text-align: left; background: #ffffff;">
        <h3 style="font-size: 1.4rem; font-weight: 700; margin-bottom: 0.5rem;">🔒 Acceso Administrador</h3>
        <p style="font-size: 0.95rem; color: var(--text-muted); margin-bottom: 1.2rem;">Ingrese la clave de seguridad PIN:</p>
        
        {#if pinError}
          <div style="color: var(--accent-red); font-size: 0.9rem; font-weight: 600; margin-bottom: 1rem;">
            {pinError}
          </div>
        {/if}

        <form on:submit|preventDefault={verifyAdminPin}>
          <input 
            type="password" 
            class="minimal-input" 
            bind:value={adminPinInput} 
            placeholder="Clave PIN (7612)" 
            style="margin-bottom: 1.5rem; letter-spacing: 4px; font-size: 1.3rem; text-align: center;"
          />
          <div style="display: flex; gap: 0.8rem;">
            <button type="submit" class="minimal-btn" style="flex: 1;">Acceder</button>
            <button type="button" class="minimal-btn" on:click={() => (showPinModal = false)} style="flex: 1; background: #e2e8f0; color: var(--text-main);">Cancelar</button>
          </div>
        </form>
      </div>
    </div>
  {/if}

  <!-- Settings Modal (Secured with PIN) -->
  {#if showSettingsModal}
    <div style="position: fixed; inset: 0; background: rgba(15, 23, 42, 0.6); backdrop-filter: blur(8px); display: flex; align-items: center; justify-content: center; z-index: 130; padding: 1.5rem;">
      <div class="card" style="max-width: 480px; text-align: left; background: #ffffff;">
        <h2 style="font-size: 1.6rem; font-weight: 700; margin-bottom: 1.5rem;">⚙️ Configuración Administrador</h2>
        
        <div style="margin-bottom: 1.2rem;">
          <label for="serverHostInput" style="display: block; font-size: 0.9rem; color: var(--text-muted); margin-bottom: 0.5rem;">IP / URL Servidor Local:</label>
          <input 
            id="serverHostInput"
            type="text" 
            class="minimal-input"
            bind:value={serverHost} 
            placeholder="http://192.168.1.9:8080" 
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
            Guardar y Sincronizar
          </button>
          <button class="minimal-btn" on:click={() => (showSettingsModal = false)} style="flex: 1; background: #e2e8f0; color: var(--text-main);">
            Cerrar
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>
