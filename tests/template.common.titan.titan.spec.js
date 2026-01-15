import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import path from "path";

// Mock de fs
vi.mock("fs", () => ({
  default: {
    existsSync: vi.fn(),
    mkdirSync: vi.fn(),
    writeFileSync: vi.fn(),
  },
}));

// Mock de bundle
vi.mock("../templates/common/titan/bundle.js", () => ({
  bundle: vi.fn().mockResolvedValue(undefined),
}));

// Import after mocks
import fs from "fs";
import { bundle } from "../templates/common/titan/bundle.js";

// Importar el módulo bajo prueba
// NOTA: Necesitas exportar las funciones internas para testing completo
import t, {
  cyan,
  green,
  addRoute,
  getRoutes,
  getDynamicRoutes,
  getActionMap,
  resetRoutes,
} from "../templates/common/titan/titan.js";

describe("t.js (Titan App Builder)", () => {
  const root = process.cwd();

  beforeEach(() => {
    vi.clearAllMocks();
    resetRoutes(); // Limpiar estado entre tests
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe("Color functions", () => {
    it("cyan should wrap text with cyan ANSI codes", () => {
      expect(cyan("test")).toBe("\x1b[36mtest\x1b[0m");
    });

    it("green should wrap text with green ANSI codes", () => {
      expect(green("test")).toBe("\x1b[32mtest\x1b[0m");
    });
  });

  describe("addRoute()", () => {
    it("should return an object with reply and action methods", () => {
      const handler = addRoute("GET", "/test");
      
      expect(handler).toHaveProperty("reply");
      expect(handler).toHaveProperty("action");
      expect(typeof handler.reply).toBe("function");
      expect(typeof handler.action).toBe("function");
    });

    describe("reply()", () => {
      it("should store text response for string values", () => {
        const handler = addRoute("GET", "/hello");
        handler.reply("Hello World");

        const routes = getRoutes();
        expect(routes["GET:/hello"]).toEqual({
          type: "text",
          value: "Hello World"
        });
      });

      it("should store json response for object values", () => {
        const handler = addRoute("GET", "/data");
        handler.reply({ message: "Hello", count: 42 });

        const routes = getRoutes();
        expect(routes["GET:/data"]).toEqual({
          type: "json",
          value: { message: "Hello", count: 42 }
        });
      });

      it("should store json response for array values", () => {
        const handler = addRoute("GET", "/list");
        handler.reply([1, 2, 3]);

        const routes = getRoutes();
        expect(routes["GET:/list"]).toEqual({
          type: "json",
          value: [1, 2, 3]
        });
      });

      it("should handle number values as text", () => {
        const handler = addRoute("GET", "/number");
        handler.reply(42);

        const routes = getRoutes();
        expect(routes["GET:/number"]).toEqual({
          type: "text",
          value: 42
        });
      });
    });

    describe("action()", () => {
      it("should store action for static routes", () => {
        const handler = addRoute("POST", "/submit");
        handler.action("submitForm");

        const routes = getRoutes();
        const actionMap = getActionMap();

        expect(routes["POST:/submit"]).toEqual({
          type: "action",
          value: "submitForm"
        });
        expect(actionMap["POST:/submit"]).toBe("submitForm");
      });

      it("should store dynamic route with params", () => {
        const handler = addRoute("GET", "/users/:id");
        handler.action("getUser");

        const dynamicRoutes = getDynamicRoutes();
        
        expect(dynamicRoutes["GET"]).toContainEqual({
          method: "GET",
          pattern: "/users/:id",
          action: "getUser"
        });
      });

      it("should store dynamic route with multiple params", () => {
        const handler = addRoute("GET", "/users/:userId/posts/:postId");
        handler.action("getUserPost");

        const dynamicRoutes = getDynamicRoutes();
        
        expect(dynamicRoutes["GET"]).toContainEqual({
          method: "GET",
          pattern: "/users/:userId/posts/:postId",
          action: "getUserPost"
        });
      });

      it("should not add to actionMap for dynamic routes", () => {
        const handler = addRoute("GET", "/items/:id");
        handler.action("getItem");

        const actionMap = getActionMap();
        
        expect(actionMap["GET:/items/:id"]).toBeUndefined();
      });
    });
  });

  describe("t.get()", () => {
    it("should create a GET route handler", () => {
      const handler = t.get("/api/users");
      
      expect(handler).toHaveProperty("reply");
      expect(handler).toHaveProperty("action");
    });

    it("should register GET route with reply", () => {
      t.get("/api/health").reply({ status: "ok" });

      const routes = getRoutes();
      expect(routes["GET:/api/health"]).toEqual({
        type: "json",
        value: { status: "ok" }
      });
    });

    it("should register GET route with action", () => {
      t.get("/api/data").action("fetchData");

      const routes = getRoutes();
      expect(routes["GET:/api/data"]).toEqual({
        type: "action",
        value: "fetchData"
      });
    });
  });

  describe("t.post()", () => {
    it("should create a POST route handler", () => {
      const handler = t.post("/api/submit");
      
      expect(handler).toHaveProperty("reply");
      expect(handler).toHaveProperty("action");
    });

    it("should register POST route with reply", () => {
      t.post("/api/echo").reply("received");

      const routes = getRoutes();
      expect(routes["POST:/api/echo"]).toEqual({
        type: "text",
        value: "received"
      });
    });

    it("should register POST route with action", () => {
      t.post("/api/create").action("createItem");

      const routes = getRoutes();
      const actionMap = getActionMap();

      expect(routes["POST:/api/create"]).toEqual({
        type: "action",
        value: "createItem"
      });
      expect(actionMap["POST:/api/create"]).toBe("createItem");
    });
  });

  describe("t.log()", () => {
    it("should log message with module prefix", () => {
      const consoleSpy = vi.spyOn(console, "log").mockImplementation(() => {});

      t.log("MyModule", "Hello World");

      expect(consoleSpy).toHaveBeenCalledWith(
        expect.stringContaining("MyModule")
      );
      expect(consoleSpy).toHaveBeenCalledWith(
        expect.stringContaining("Hello World")
      );
    });

    it("should format module name with magenta color", () => {
      const consoleSpy = vi.spyOn(console, "log").mockImplementation(() => {});

      t.log("Test", "message");

      const call = consoleSpy.mock.calls[0][0];
      expect(call).toContain("\x1b[35m"); // Magenta color code
    });
  });

  describe("t.start()", () => {
    beforeEach(() => {
      vi.spyOn(console, "log").mockImplementation(() => {});
      vi.spyOn(console, "error").mockImplementation(() => {});
      vi.spyOn(process, "exit").mockImplementation(() => {});
    });

    it("should call bundle() to prepare runtime", async () => {
      vi.mocked(fs.existsSync).mockReturnValue(true);

      await t.start();

      expect(bundle).toHaveBeenCalled();
    });

    it("should create server directory if it doesn't exist", async () => {
      vi.mocked(fs.existsSync).mockReturnValue(false);

      await t.start();

      expect(fs.mkdirSync).toHaveBeenCalledWith(
        path.join(root, "server"),
        { recursive: true }
      );
    });

    it("should not create server directory if it exists", async () => {
      vi.mocked(fs.existsSync).mockReturnValue(true);

      await t.start();

      expect(fs.mkdirSync).not.toHaveBeenCalled();
    });

    it("should write routes.json with correct structure", async () => {
      vi.mocked(fs.existsSync).mockReturnValue(true);
      
      // Setup some routes
      t.get("/test").reply("hello");
      t.get("/users/:id").action("getUser");

      await t.start(8080);

      const writeFileCalls = vi.mocked(fs.writeFileSync).mock.calls;
      const routesCall = writeFileCalls.find(call => 
        call[0].toString().includes("routes.json")
      );

      expect(routesCall).toBeDefined();
      
      const routesJson = JSON.parse(routesCall[1]);
      expect(routesJson.__config).toEqual({ port: 8080 });
      expect(routesJson.routes).toBeDefined();
      expect(routesJson.__dynamic_routes).toBeDefined();
    });

    it("should write action_map.json", async () => {
      vi.mocked(fs.existsSync).mockReturnValue(true);
      
      t.post("/submit").action("handleSubmit");

      await t.start();

      const writeFileCalls = vi.mocked(fs.writeFileSync).mock.calls;
      const actionMapCall = writeFileCalls.find(call => 
        call[0].toString().includes("action_map.json")
      );

      expect(actionMapCall).toBeDefined();
      
      const actionMapJson = JSON.parse(actionMapCall[1]);
      expect(actionMapJson["POST:/submit"]).toBe("handleSubmit");
    });

    it("should use default port 3000", async () => {
      vi.mocked(fs.existsSync).mockReturnValue(true);

      await t.start();

      const writeFileCalls = vi.mocked(fs.writeFileSync).mock.calls;
      const routesCall = writeFileCalls.find(call => 
        call[0].toString().includes("routes.json")
      );

      const routesJson = JSON.parse(routesCall[1]);
      expect(routesJson.__config.port).toBe(3000);
    });

    it("should log success message", async () => {
      vi.mocked(fs.existsSync).mockReturnValue(true);
      const consoleSpy = vi.spyOn(console, "log");

      await t.start();

      expect(consoleSpy).toHaveBeenCalledWith(
        expect.stringContaining("Titan metadata written successfully")
      );
    });

    it("should log custom message if provided", async () => {
      vi.mocked(fs.existsSync).mockReturnValue(true);
      const consoleSpy = vi.spyOn(console, "log");

      await t.start(3000, "Server starting on port 3000");

      expect(consoleSpy).toHaveBeenCalledWith(
        expect.stringContaining("Server starting on port 3000")
      );
    });

    it("should handle bundle errors gracefully", async () => {
      vi.mocked(bundle).mockRejectedValueOnce(new Error("Bundle failed"));
      const consoleSpy = vi.spyOn(console, "error");
      const exitSpy = vi.spyOn(process, "exit");

      await t.start();

      expect(consoleSpy).toHaveBeenCalledWith(
        expect.stringContaining("Build Error")
      );
      expect(exitSpy).toHaveBeenCalledWith(1);
    });

    it("should handle fs.writeFileSync errors", async () => {
      vi.mocked(fs.existsSync).mockReturnValue(true);
      vi.mocked(fs.writeFileSync).mockImplementationOnce(() => {
        throw new Error("Permission denied");
      });
      const consoleSpy = vi.spyOn(console, "error");
      const exitSpy = vi.spyOn(process, "exit");

      await t.start();

      expect(consoleSpy).toHaveBeenCalledWith(
        expect.stringContaining("Build Error")
      );
      expect(exitSpy).toHaveBeenCalledWith(1);
    });

    it("should include dynamic routes in routes.json", async () => {
      vi.mocked(fs.existsSync).mockReturnValue(true);
      
      t.get("/posts/:id").action("getPost");
      t.post("/users/:userId/comments").action("addComment");

      await t.start();

      const writeFileCalls = vi.mocked(fs.writeFileSync).mock.calls;
      const routesCall = writeFileCalls.find(call => 
        call[0].toString().includes("routes.json")
      );

      const routesJson = JSON.parse(routesCall[1]);
      
      expect(routesJson.__dynamic_routes).toContainEqual({
        method: "GET",
        pattern: "/posts/:id",
        action: "getPost"
      });
      expect(routesJson.__dynamic_routes).toContainEqual({
        method: "POST",
        pattern: "/users/:userId/comments",
        action: "addComment"
      });
    });
  });

  describe("Integration: Multiple routes", () => {
    it("should handle mixed static and dynamic routes", () => {
      t.get("/").reply("Welcome");
      t.get("/api/health").reply({ status: "ok" });
      t.get("/users/:id").action("getUser");
      t.post("/users").action("createUser");
      t.post("/users/:id/update").action("updateUser");

      const routes = getRoutes();
      const dynamicRoutes = getDynamicRoutes();
      const actionMap = getActionMap();

      // Static routes
      expect(routes["GET:/"]).toEqual({ type: "text", value: "Welcome" });
      expect(routes["GET:/api/health"]).toEqual({ type: "json", value: { status: "ok" } });
      expect(routes["POST:/users"]).toEqual({ type: "action", value: "createUser" });

      // Dynamic routes
      expect(dynamicRoutes["GET"]).toHaveLength(1);
      expect(dynamicRoutes["POST"]).toHaveLength(1);

      // Action map (only static actions)
      expect(actionMap["POST:/users"]).toBe("createUser");
      expect(actionMap["GET:/users/:id"]).toBeUndefined();
    });
  });
});