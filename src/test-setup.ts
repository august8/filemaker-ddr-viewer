import "@testing-library/jest-dom";

// jsdom は scrollIntoView を実装していないためモック
Element.prototype.scrollIntoView = vi.fn();
