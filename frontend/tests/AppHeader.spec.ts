import { mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";

import AppHeader from "../src/components/AppHeader/AppHeader.vue";
import * as backend from "../src/services/backend";

vi.mock("../src/services/backend", () => ({ selectInstallationDirectory: vi.fn() }));

const baseProps = {
  projectPath: "/home/user/current-project",
  scope: "project" as const,
};

describe("AppHeader installation destination", () => {
  beforeEach(() => vi.clearAllMocks());

  it("shows the application logo before its name", () => {
    const wrapper = mount(AppHeader, { props: baseProps });
    const brand = wrapper.get(".app-header__brand");
    expect(brand.find("img.app-header__logo").exists()).toBe(true);
    expect(brand.get("h1").text()).toBe("Skills Installer");
  });

  it("opens the native directory picker and emits the selected folder", async () => {
    vi.mocked(backend.selectInstallationDirectory).mockResolvedValue("/home/user/new-project");
    const wrapper = mount(AppHeader, { props: baseProps });

    expect(wrapper.find("input").exists()).toBe(false);
    await wrapper.get('[aria-label="Select project installation folder"]').trigger("click");

    expect(backend.selectInstallationDirectory).toHaveBeenCalledWith("/home/user/current-project");
    expect(wrapper.emitted("update:projectPath")).toEqual([["/home/user/new-project"]]);
  });

  it("keeps the current destination when the native picker is cancelled", async () => {
    vi.mocked(backend.selectInstallationDirectory).mockResolvedValue(null);
    const wrapper = mount(AppHeader, { props: baseProps });

    await wrapper.get('[aria-label="Select project installation folder"]').trigger("click");
    expect(wrapper.emitted("update:projectPath")).toBeUndefined();
  });
});

describe("AppHeader scope switch", () => {
  beforeEach(() => vi.clearAllMocks());

  it("shows the folder picker in Project mode and emits update:scope when Global is clicked", async () => {
    const wrapper = mount(AppHeader, { props: baseProps });

    expect(wrapper.find(".app-header__path").exists()).toBe(true);
    expect(wrapper.find(".app-header__global-badge").exists()).toBe(false);
    expect(wrapper.get('button[title="Install into the selected project folder"]').attributes("aria-pressed")).toBe(
      "true",
    );

    await wrapper.get('button[title="Install into your home directory, for every project"]').trigger("click");
    expect(wrapper.emitted("update:scope")).toEqual([["global"]]);
  });

  it("swaps the folder picker for a static Global destination and shows the badge in Global mode", () => {
    const wrapper = mount(AppHeader, { props: { ...baseProps, scope: "global" } });

    expect(wrapper.find(".app-header__path").exists()).toBe(false);
    expect(wrapper.get(".app-header__global-badge").text()).toBe("Global");
    expect(wrapper.get('button[title="Install into your home directory, for every project"]').attributes(
      "aria-pressed",
    )).toBe("true");
    expect(wrapper.get(".app-header").classes()).toContain("app-header--global");
  });
});
