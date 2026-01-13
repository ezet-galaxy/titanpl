// Ejemplo: Prueba asíncrona (si tu módulo tuviera funciones async)
describe('Funciones asíncronas', () => {
  it('resuelve una promesa correctamente', async () => {
    const result = await Promise.resolve(42);
    expect(result).toBe(42);
  });
});

// Ejemplo: Mock de una función
describe('Usando mocks', () => {
  it('llama a una función mockeada', () => {
    const mockFn = vi.fn();
    mockFn('test');
    expect(mockFn).toHaveBeenCalledWith('test');
  });
});