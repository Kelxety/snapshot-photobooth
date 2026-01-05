<script lang="ts">
  import { goto } from "$app/navigation";

  let email = $state("");
  let password = $state("");
  let errorMsg = $state("");
  let isLoading = $state(false);

  async function handleSignIn(event: Event) {
    event.preventDefault();
    errorMsg = "";
    
    // Basic validation
    if (!email || !password) {
      errorMsg = "Please fill in all fields";
      return;
    }

    if (!email.includes("@")) {
      errorMsg = "Please enter a valid email address";
      return;
    }

    isLoading = true;

    try {
      // Temporary hardcoded credentials
      const TEMP_EMAIL = "snapshot@gmail.com";
      const TEMP_PASSWORD = "password";
      
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      // Check credentials
      if (email === TEMP_EMAIL && password === TEMP_PASSWORD) {
        // On success, redirect to main page
        goto("/home");
      } else {
        errorMsg = "Invalid email or password. Try snapshot@gmail.com / password";
        isLoading = false;
      }
    } catch (error) {
      errorMsg = "Invalid credentials. Please try again.";
      console.error("Sign in error:", error);
      isLoading = false;
    }
  }
</script>

<main class="container">
  <div class="signin-card">
    <div class="header">
      <h1>Snapshot PhotoBooth</h1>
      <p class="subtitle">Sign in to start your photo session</p>
    </div>

    <form class="signin-form" onsubmit={handleSignIn}>
      <div class="form-group">
        <label for="email">Email</label>
        <input
          id="email"
          type="email"
          placeholder="Enter your email"
          bind:value={email}
          disabled={isLoading}
          required
        />
      </div>

      <div class="form-group">
        <label for="password">Password</label>
        <input
          id="password"
          type="password"
          placeholder="Enter your password"
          bind:value={password}
          disabled={isLoading}
          required
        />
      </div>

      {#if errorMsg}
        <div class="error-message">{errorMsg}</div>
      {/if}

      <button type="submit" class="signin-button" disabled={isLoading}>
        {isLoading ? "Signing in..." : "Sign In"}
      </button>

      <div class="footer-links">
        <a href="/forgot-password">Forgot password?</a>
        <span class="separator">•</span>
        <a href="/signup">Create account</a>
      </div>
    </form>
  </div>
</main>

<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  text-align: center;
  padding: 20px;
}

.signin-card {
  background: white;
  border-radius: 12px;
  padding: 40px;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  max-width: 400px;
  width: 100%;
}

.header {
  margin-bottom: 30px;
}

.header h1 {
  font-size: 28px;
  margin: 0 0 10px 0;
  color: #0f0f0f;
}

.subtitle {
  color: #666;
  font-size: 14px;
  margin: 0;
}

.signin-form {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.form-group {
  display: flex;
  flex-direction: column;
  text-align: left;
  gap: 8px;
}

.form-group label {
  font-weight: 500;
  font-size: 14px;
  color: #333;
}

.form-group input {
  padding: 12px 16px;
  border: 2px solid #e0e0e0;
  border-radius: 8px;
  font-size: 16px;
  transition: all 0.2s ease;
  outline: none;
}

.form-group input:focus {
  border-color: #646cff;
  box-shadow: 0 0 0 3px rgba(100, 108, 255, 0.1);
}

.form-group input:disabled {
  background-color: #f5f5f5;
  cursor: not-allowed;
  opacity: 0.6;
}

.error-message {
  padding: 12px;
  background-color: #fee;
  border: 1px solid #fcc;
  border-radius: 6px;
  color: #c33;
  font-size: 14px;
  text-align: center;
}

.signin-button {
  padding: 14px;
  background-color: #646cff;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
  margin-top: 10px;
}

.signin-button:hover:not(:disabled) {
  background-color: #535bf2;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(100, 108, 255, 0.3);
}

.signin-button:active:not(:disabled) {
  transform: translateY(0);
}

.signin-button:disabled {
  background-color: #ccc;
  cursor: not-allowed;
  opacity: 0.6;
}

.footer-links {
  margin-top: 20px;
  font-size: 14px;
  color: #666;
}

.footer-links a {
  color: #646cff;
  text-decoration: none;
  font-weight: 500;
  transition: color 0.2s ease;
}

.footer-links a:hover {
  color: #535bf2;
  text-decoration: underline;
}

.separator {
  margin: 0 8px;
  color: #ccc;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  .signin-card {
    background: #1a1a1a;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
  }

  .header h1 {
    color: #f6f6f6;
  }

  .subtitle {
    color: #a0a0a0;
  }

  .form-group label {
    color: #d0d0d0;
  }

  .form-group input {
    background-color: #2a2a2a;
    border-color: #444;
    color: #f6f6f6;
  }

  .form-group input:focus {
    border-color: #7c84ff;
    box-shadow: 0 0 0 3px rgba(124, 132, 255, 0.2);
    background-color: #2f2f2f;
  }

  .form-group input:disabled {
    background-color: #1f1f1f;
    color: #888;
  }

  .error-message {
    background-color: #3a1a1a;
    border-color: #5a2a2a;
    color: #ff6b6b;
  }

  .signin-button:disabled {
    background-color: #3a3a3a;
    color: #888;
  }

  .footer-links {
    color: #a0a0a0;
  }

  .footer-links a {
    color: #7c84ff;
  }

  .footer-links a:hover {
    color: #9ca4ff;
  }

  .separator {
    color: #555;
  }
}
</style>
