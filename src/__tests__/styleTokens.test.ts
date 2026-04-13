import { describe, it, expect } from "vitest";
import {
  BADGE_BASE,
  BADGE_VARIANTS,
  DIFF_BADGE_VARIANTS,
  SECTION_HEADER,
  MAIN_HEADER,
  CODE_BLOCK,
  CARD,
  LIST_ROW,
  SELECT_INPUT,
} from "../styles/tokens";

describe("BADGE_BASE", () => {
  it("contains text-xs", () => expect(BADGE_BASE).toContain("text-xs"));
  it("contains font-medium", () => expect(BADGE_BASE).toContain("font-medium"));
  it("contains rounded (not rounded-full)", () => {
    expect(BADGE_BASE).toContain("rounded");
    expect(BADGE_BASE).not.toContain("rounded-full");
  });
  it("contains px-2", () => expect(BADGE_BASE).toContain("px-2"));
  it("contains py-0.5", () => expect(BADGE_BASE).toContain("py-0.5"));
});

describe("BADGE_VARIANTS", () => {
  it("has all required color keys", () => {
    expect(BADGE_VARIANTS).toHaveProperty("blue");
    expect(BADGE_VARIANTS).toHaveProperty("purple");
    expect(BADGE_VARIANTS).toHaveProperty("green");
    expect(BADGE_VARIANTS).toHaveProperty("yellow");
    expect(BADGE_VARIANTS).toHaveProperty("red");
    expect(BADGE_VARIANTS).toHaveProperty("gray");
  });

  it("each variant contains BADGE_BASE classes", () => {
    for (const variant of Object.values(BADGE_VARIANTS)) {
      expect(variant).toContain("text-xs");
      expect(variant).toContain("font-medium");
      expect(variant).toContain("px-2");
    }
  });
});

describe("DIFF_BADGE_VARIANTS", () => {
  it("has Added / Removed / Modified keys", () => {
    expect(DIFF_BADGE_VARIANTS).toHaveProperty("Added");
    expect(DIFF_BADGE_VARIANTS).toHaveProperty("Removed");
    expect(DIFF_BADGE_VARIANTS).toHaveProperty("Modified");
  });
});

describe("SECTION_HEADER", () => {
  it("contains text-sm", () => expect(SECTION_HEADER).toContain("text-sm"));
  it("contains font-semibold", () => expect(SECTION_HEADER).toContain("font-semibold"));
  it("contains mb-2", () => expect(SECTION_HEADER).toContain("mb-2"));
});

describe("MAIN_HEADER", () => {
  it("contains text-lg", () => expect(MAIN_HEADER).toContain("text-lg"));
  it("contains font-bold", () => expect(MAIN_HEADER).toContain("font-bold"));
  it("contains mb-4", () => expect(MAIN_HEADER).toContain("mb-4"));
});

describe("CODE_BLOCK", () => {
  it("contains font-mono", () => expect(CODE_BLOCK).toContain("font-mono"));
  it("contains bg-gray-50", () => expect(CODE_BLOCK).toContain("bg-gray-50"));
  it("contains border-gray-200", () => expect(CODE_BLOCK).toContain("border-gray-200"));
  it("contains text-xs", () => expect(CODE_BLOCK).toContain("text-xs"));
});

describe("CARD", () => {
  it("contains border border-gray-200", () => expect(CARD).toContain("border border-gray-200"));
  it("contains rounded-xl", () => expect(CARD).toContain("rounded-xl"));
});

describe("LIST_ROW", () => {
  it("contains hover:bg-blue-50", () => expect(LIST_ROW).toContain("hover:bg-blue-50"));
  it("contains px-3 py-2", () => expect(LIST_ROW).toContain("px-3 py-2"));
});

describe("SELECT_INPUT", () => {
  it("contains border border-gray-300", () => expect(SELECT_INPUT).toContain("border border-gray-300"));
});
