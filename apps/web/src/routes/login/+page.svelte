<script lang="ts">
  import { goto } from '$app/navigation';
  import { ProgressRadial } from '@skeletonlabs/skeleton';
  import { auth } from '$lib/stores/auth';

  let username = $state('');
  let password = $state('');
  let isSubmitting = $state(false);

  async function handleSubmit(e: Event) {
    e.preventDefault();
    isSubmitting = true;

    const success = await auth.login(username, password);

    if (success) {
      goto('/');
    }

    isSubmitting = false;
  }
</script>

<svelte:head>
  <title>Login - JejakCuan</title>
</svelte:head>

<div class="flex items-center justify-center min-h-[80vh]">
  <div class="card p-8 w-full max-w-md">
    <header class="card-header text-center">
      <h1 class="h2">JejakCuan</h1>
      <p class="text-surface-600-300-token">Indonesian Stock Tracker</p>
    </header>

    <form onsubmit={handleSubmit} class="space-y-4 mt-6">
      {#if $auth.error}
        <aside class="alert variant-filled-error">
          <div class="alert-message">
            <p>{$auth.error}</p>
          </div>
        </aside>
      {/if}

      <label class="label">
        <span>Username</span>
        <input
          type="text"
          bind:value={username}
          class="input"
          placeholder="Enter username"
          required
          disabled={isSubmitting}
        />
      </label>

      <label class="label">
        <span>Password</span>
        <input
          type="password"
          bind:value={password}
          class="input"
          placeholder="Enter password"
          required
          disabled={isSubmitting}
        />
      </label>

      <button type="submit" class="btn variant-filled-primary w-full" disabled={isSubmitting}>
        {#if isSubmitting}
          <ProgressRadial width="w-5" stroke={100} meter="stroke-on-primary-token" track="stroke-on-primary-token/30" />
          <span>Signing in...</span>
        {:else}
          Sign In
        {/if}
      </button>
    </form>

    <footer class="card-footer text-center text-sm text-surface-600-300-token mt-4">
      <p>Default: admin / admin123</p>
    </footer>
  </div>
</div>
