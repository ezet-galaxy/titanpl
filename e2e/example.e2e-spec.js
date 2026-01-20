import { describe, it, expect } from "vitest";

describe("Sanity Check", () => {
    it("should pass a simple test", () => {
        expect(1 + 1).toBe(2);
    });

    it("should verify test environment is working", () => {
        expect(process.version).toMatch(/^v\d+/);
        expect(typeof process.cwd()).toBe("string");
    });
});