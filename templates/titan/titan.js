import fs from "fs";
import path from "path";
import { bundle } from "./bundle.js";

const cyan = (t) => `\x1b[36m${t}\x1b[0m`;
const green = (t) => `\x1b[32m${t}\x1b[0m`;

const routes = {};
const dynamicRoutes = {};
const actionMap = {};

/**
 * @param {string} method
 * @param {string} route
 * @returns {TitanRouteBuilder}
 */
function addRoute(method, route) {
  const key = `${method.toUpperCase()}:${route}`;

  return {
    reply(value) {
      routes[key] = {
        type: typeof value === "object" ? "json" : "text",
        value,
      };
    },

    action(name) {
      if (route.includes(":")) {
        if (!dynamicRoutes[method]) dynamicRoutes[method] = [];
        dynamicRoutes[method].push({
          method: method.toUpperCase(),
          pattern: route,
          action: name,
        });
      } else {
        routes[key] = {
          type: "action",
          value: name,
        };
        actionMap[key] = name;
      }
    },
  };
}

// Build time methods implementation
const buildTimeMethods = {
  get(route) {
    return addRoute("GET", route);
  },

  post(route) {
    return addRoute("POST", route);
  },

  put(route) {
    return addRoute("PUT", route);
  },

  delete(route) {
    return addRoute("DELETE", route);
  },

  patch(route) {
    return addRoute("PATCH", route);
  },

  log(module, msg) {
    console.log(`[\x1b[35m${module}\x1b[0m] ${msg}`);
  },

  async start(port = 3000, msg = "") {
    try {
      console.log(cyan("[Titan] Preparing runtime..."));
      await bundle();

      const base = path.join(process.cwd(), "server");
      if (!fs.existsSync(base)) {
        fs.mkdirSync(base, { recursive: true });
      }

      const routesPath = path.join(base, "routes.json");
      const actionMapPath = path.join(base, "action_map.json");

      fs.writeFileSync(
        routesPath,
        JSON.stringify(
          {
            __config: { port },
            routes,
            __dynamic_routes: Object.values(dynamicRoutes).flat(),
          },
          null,
          2
        )
      );

      fs.writeFileSync(actionMapPath, JSON.stringify(actionMap, null, 2));

      console.log(green("✔ Titan metadata written successfully"));
      if (msg) console.log(cyan(msg));
    } catch (e) {
      console.error(`\x1b[31m[Titan] Build Error: ${e.message}\x1b[0m`);
      process.exit(1);
    }
  },
};

// Proxy to catch any undefined method access
const t = /** @type {TitanRuntime} */ (new Proxy(buildTimeMethods, {
  get(target, prop) {
    if (prop in target) {
      return target[prop];
    }

    return new Proxy(() => {
      throw new Error(`[Titan] t.${String(prop)}() is only available inside actions at runtime.`);
    }, {
      get(_, nestedProp) {
        return () => {
          throw new Error(`[Titan] t.${String(prop)}.${String(nestedProp)}() is only available inside actions at runtime.`);
        };
      }
    });
  }
}));

// @ts-ignore - t is assigned to globalThis for runtime
globalThis.t = t;
export default t;