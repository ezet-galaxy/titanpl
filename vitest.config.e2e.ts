import { defineConfig } from "vitest/config";

export default defineConfig({
    test: {
        globals: true,
        environment: "node",
        include: ["e2e/**/*.e2e-spec.{js,ts}"],
        // E2E tests are slower, need more time
        testTimeout: 60000,
        hookTimeout: 60000,
        // Run tests sequentially to avoid port conflicts
        sequence: {
            concurrent: false,
        },
        // Separate coverage for e2e
        coverage: {
            provider: "v8",
            reporter: ["text", "json", "html"],
            reportsDirectory: "./coverage-e2e",
            include: ["index.js", "titan/**/*.js"],
            exclude: ["tests/**", "node_modules/**"],
        },
    },
});