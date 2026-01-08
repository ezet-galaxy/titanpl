
# 🛠️ Titan SDK

**Empower your Titan Planet development with first-class type safety and elite developer tools.**

[![npm version](https://img.shields.io/npm/v/titan-sdk.svg?style=flat-square)](https://www.npmjs.com/package/titan-sdk)
[![License: ISC](https://img.shields.io/badge/License-ISC-blue.svg?style=flat-square)](https://opensource.org/licenses/ISC)

---

## 🌌 Overview

**Titan SDK** is the official developer toolkit for [Titan Planet](https://github.com/ezetgalaxy/titan). It provides the glue between your JavaScript actions and the high-performance Rust runtime, ensuring you have full IntelliSense, type safety, and debugging capabilities.

Whether you are building standard API actions or complex extensions, Titan SDK is your essential companion.

---

## ✨ Features

- **💎 Elite IntelliSense**: Full TypeScript definitions for the global `t` object.
- **🛡️ Type Safety**: Prevent runtime errors with compile-time checks for `t.log`, `t.fetch`, `t.db`, and more.
- **🔌 Extension Support**: Tools and types for building custom extensions that plug directly into the Titan Rust engine.
- **🚀 Zero Overhead**: Designed to be a development-only dependency that maximizes productivity without bloating your production binary.

---

## 🚀 Getting Started

### Installation

Add the SDK to your Titan project:

```bash
npm install --save-dev titan-sdk
```

### Enable IntelliSense

To get full autocomplete for the Titan runtime APIs, simply create a `tsconfig.json` or `jsconfig.json` in your project root:

```json
{
  "compilerOptions": {
    "types": ["titan-sdk"]
  }
}
```

Now, when you type `t.` in your actions, you'll see everything available:

```js
export function myAction(req) {
  t.log.info("Request received", req.path);
  
  // Fully typed db queries!
  const data = t.db.query("SELECT * FROM users WHERE id = $1", [req.params.id]);
  
  return data;
}
```

---

## 🔧 Core APIs Powered by SDK

The SDK provides types for the entire `t` namespace:

- **`t.log`**: Structured logging (info, warn, error).
- **`t.fetch`**: High-performance Rust-native fetch wrapper.
- **`t.db`**: Native PostgreSQL interface for extreme speed.
- **`t.read`**: Optimized file system access.
- **`t.jwt`**: Built-in JWT handling (if enabled).

---

## 🧩 Building Extensions

Titan SDK allows you to define types for your own extensions. 

```typescript
// Define your extension types
declare global {
    namespace Titan {
        interface Runtime {
            myCustomTool: {
                doSomething: () => void;
            };
        }
    }
}
```

---

## 🌍 Community & Documentation

- **Documentation**: [Titan Planet Docs](https://titan-docs-ez.vercel.app/docs)
- **Author**: [ezetgalaxy](https://github.com/ezetgalaxy)
- **Ecosystem**: [Titan Planet](https://github.com/ezetgalaxy/titan)

---

<p align="center">
  Built with ❤️ for the <a href="https://github.com/ezetgalaxy/titan">Titan Planet</a> ecosystem.
</p>
