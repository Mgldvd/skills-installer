import { mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: vi.fn().mockResolvedValue(undefined) }));
import { openUrl } from "@tauri-apps/plugin-opener";

import ProjectsDialog from "../src/components/ProjectsDialog/ProjectsDialog.vue";
import type { Project } from "../src/types";

const projects: Project[] = [
  { id: "atlas", name: "Atlas", gitUrl: "https://github.com/me/atlas", skillNames: ["triage", "docs"] },
  { id: "zephyr", name: "Zephyr", gitUrl: null, skillNames: ["triage"] },
];

describe("ProjectsDialog", () => {
  it("lists saved Projects with their git URL and Skill count", () => {
    const wrapper = mount(ProjectsDialog, {
      props: { open: true, projects, installedSkillNames: [] },
    });

    const rows = wrapper.findAll(".projects-dialog__row");
    expect(rows).toHaveLength(2);
    expect(rows[0].get(".projects-dialog__name").text()).toBe("Atlas");
    expect(rows[0].get(".projects-dialog__git-url").text()).toBe("https://github.com/me/atlas");
    expect(rows[0].get(".projects-dialog__count").text()).toBe("2 Skills");
    expect(rows[1].find(".projects-dialog__git-url").exists()).toBe(false);
    expect(rows[1].get(".projects-dialog__count").text()).toBe("1 Skill");
  });

  it("shows the GitHub or GitLab mark for a recognized host, and a generic link icon otherwise", () => {
    const wrapper = mount(ProjectsDialog, {
      props: {
        open: true,
        projects: [
          { id: "gh", name: "GH", gitUrl: "https://github.com/me/gh", skillNames: ["a"] },
          { id: "gl", name: "GL", gitUrl: "https://gitlab.com/me/gl", skillNames: ["a"] },
          { id: "other", name: "Other", gitUrl: "https://example.com/me/other", skillNames: ["a"] },
        ],
        installedSkillNames: [],
      },
    });
    const rows = wrapper.findAll(".projects-dialog__row");
    expect(rows[0].get(".projects-dialog__git-icon").html()).toContain("viewBox=\"0 0 16 16\"");
    expect(rows[1].get(".projects-dialog__git-icon").html()).toContain("viewBox=\"0 0 24 24\"");
    expect(rows[2].get(".projects-dialog__git-icon").exists()).toBe(true);
  });

  it("opens the git URL in the system browser instead of navigating the app's own window", async () => {
    const wrapper = mount(ProjectsDialog, { props: { open: true, projects, installedSkillNames: [] } });
    const link = wrapper.get(".projects-dialog__git-url");

    await link.trigger("click");

    expect(openUrl).toHaveBeenCalledWith("https://github.com/me/atlas");
  });

  it("shows the empty state when nothing has been saved yet", () => {
    const wrapper = mount(ProjectsDialog, { props: { open: true, projects: [], installedSkillNames: [] } });
    expect(wrapper.get(".projects-dialog__empty").text()).toBe("No Projects saved yet.");
  });

  it("fills the Name field from the git URL's repo name while Name is still empty", async () => {
    const wrapper = mount(ProjectsDialog, { props: { open: true, projects: [], installedSkillNames: ["a"] } });

    await wrapper.get("input[type=url]").setValue("https://github.com/Mgldvd/skills-installer");

    expect((wrapper.get("input[type=text]").element as HTMLInputElement).value).toBe("skills-installer");
  });

  it("never overwrites a Name the user already typed themselves", async () => {
    const wrapper = mount(ProjectsDialog, { props: { open: true, projects: [], installedSkillNames: ["a"] } });

    await wrapper.get("input[type=text]").setValue("Custom Name");
    await wrapper.get("input[type=url]").setValue("https://github.com/Mgldvd/skills-installer");

    expect((wrapper.get("input[type=text]").element as HTMLInputElement).value).toBe("Custom Name");
  });

  it("disables Save until there's a name and at least one installed Skill", async () => {
    const wrapper = mount(ProjectsDialog, {
      props: { open: true, projects: [], installedSkillNames: [] },
    });
    const submit = wrapper.get("button[type=submit]");
    expect(submit.attributes("disabled")).toBeDefined();
    expect(submit.text()).toBe("Save 0 installed Skills");

    await wrapper.setProps({ installedSkillNames: ["triage"] });
    expect(wrapper.get("button[type=submit]").attributes("disabled")).toBeDefined();
    expect(wrapper.get("button[type=submit]").text()).toBe("Save 1 installed Skill");

    await wrapper.get("input[type=text]").setValue("My Project");
    expect(wrapper.get("button[type=submit]").attributes("disabled")).toBeUndefined();
  });

  it("emits save with the trimmed name and a null git URL when left blank, then clears the form", async () => {
    const wrapper = mount(ProjectsDialog, {
      props: { open: true, projects: [], installedSkillNames: ["triage"] },
    });

    await wrapper.get("input[type=text]").setValue("  My Project  ");
    await wrapper.get(".projects-dialog__form").trigger("submit");

    expect(wrapper.emitted("save")).toEqual([["My Project", null]]);
    expect((wrapper.get("input[type=text]").element as HTMLInputElement).value).toBe("");
  });

  it("emits save with a trimmed git URL when provided", async () => {
    const wrapper = mount(ProjectsDialog, {
      props: { open: true, projects: [], installedSkillNames: ["triage"] },
    });

    await wrapper.get("input[type=text]").setValue("My Project");
    await wrapper.get("input[type=url]").setValue("  https://github.com/me/repo  ");
    await wrapper.get(".projects-dialog__form").trigger("submit");

    expect(wrapper.emitted("save")).toEqual([["My Project", "https://github.com/me/repo"]]);
  });

  it("emits load with the project id", async () => {
    const wrapper = mount(ProjectsDialog, { props: { open: true, projects, installedSkillNames: [] } });
    await wrapper.findAll(".projects-dialog__row")[0].get(".button").trigger("click");
    expect(wrapper.emitted("load")).toEqual([["atlas"]]);
  });

  it("asks for confirmation before emitting delete, and does nothing if declined", async () => {
    const confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(false);
    const wrapper = mount(ProjectsDialog, { props: { open: true, projects, installedSkillNames: [] } });

    await wrapper.findAll(".projects-dialog__row")[0].get(".projects-dialog__delete").trigger("click");
    expect(confirmSpy).toHaveBeenCalled();
    expect(wrapper.emitted("delete")).toBeUndefined();

    confirmSpy.mockReturnValue(true);
    await wrapper.findAll(".projects-dialog__row")[0].get(".projects-dialog__delete").trigger("click");
    expect(wrapper.emitted("delete")).toEqual([["atlas"]]);

    confirmSpy.mockRestore();
  });

  it("shows a save error when the backend rejects it", () => {
    const wrapper = mount(ProjectsDialog, {
      props: { open: true, projects: [], installedSkillNames: [], error: "Project name cannot be empty" },
    });
    expect(wrapper.get(".projects-dialog__error").text()).toBe("Project name cannot be empty");
  });
});
