import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";

import AddSkillDialog from "../src/components/AddSkillDialog/AddSkillDialog.vue";
import { makeGroup, makeSkill } from "./fixtures";

vi.mock("../src/services/backend", () => ({
  previewSkillUrl: vi.fn(),
  previewPackUrl: vi.fn(),
}));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: vi.fn().mockResolvedValue(undefined) }));

import { openUrl } from "@tauri-apps/plugin-opener";

import * as backend from "../src/services/backend";

describe("AddSkillDialog", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("places a clickable link to browse Skills.sh beside the URL field", () => {
    const wrapper = mount(AddSkillDialog, { props: { open: true, groups: [makeGroup()] } });
    expect(wrapper.get("h2").text()).toBe("Add Skill");
    const link = wrapper.get(".add-skill-dialog__browse");
    expect(link.text()).toBe("Browse Skills.sh");
    expect(link.attributes()).toMatchObject({
      href: "https://skills.sh",
      target: "_blank",
      rel: "noopener noreferrer",
    });
  });

  it("shows a detected-skill preview once the URL resolves", async () => {
    vi.mocked(backend.previewSkillUrl).mockResolvedValue({
      canonicalUrl: "https://www.skills.sh/mattpocock/skills/triage",
      owner: "mattpocock",
      repository: "skills",
      skillName: "triage",
      repositoryUrl: "https://github.com/mattpocock/skills",
    });

    const wrapper = mount(AddSkillDialog, {
      props: { open: true, groups: [makeGroup()] },
    });

    await wrapper.find('input[type="url"]').setValue("https://www.skills.sh/mattpocock/skills/triage");
    await wrapper.find('input[type="url"]').trigger("input");
    await vi.waitFor(() => expect(backend.previewSkillUrl).toHaveBeenCalled());
    await flushPromises();

    expect(wrapper.text()).toContain("triage");
    expect(wrapper.text()).toContain("mattpocock/skills");
    expect((wrapper.find('input[type="text"]').element as HTMLInputElement).value).toBe("triage");
    expect((wrapper.find("textarea").element as HTMLTextAreaElement).value).toBe("mattpocock/skills");
    expect(wrapper.find(".add-skill-dialog__error").exists()).toBe(false);
  });

  it("detects a Pack URL and emits a catalog import instead of a Skill install", async () => {
    vi.mocked(backend.previewPackUrl).mockResolvedValue({
      canonicalUrl: "https://skills.sh/p/example123",
      suggestedName: "Pack example123",
      skills: [
        { name: "bash-defensive-patterns", description: "Defensive Bash." },
        { name: "bash-scripting", description: "Bash workflows." },
      ],
    });
    const wrapper = mount(AddSkillDialog, { props: { open: true, groups: [makeGroup({ id: "other" })] } });

    await wrapper.find('input[type="url"]').setValue("https://skills.sh/p/example123");
    await vi.waitFor(() => expect(backend.previewPackUrl).toHaveBeenCalled());
    await flushPromises();

    expect(wrapper.text()).toContain("Detected Pack: 2 Skills");
    expect(wrapper.text()).toContain("Nothing is installed into the current project");
    expect(wrapper.get('button[type="submit"]').text()).toBe("Import Pack");
    await wrapper.find("form").trigger("submit");
    expect(wrapper.emitted("importPack")?.[0]?.[0]).toMatchObject({
      url: "https://skills.sh/p/example123",
      packName: "Pack example123",
      groupId: "other",
    });
    expect(wrapper.emitted("submit")).toBeUndefined();
  });

  it("keeps detected suggestions editable", async () => {
    vi.mocked(backend.previewSkillUrl).mockResolvedValue({
      canonicalUrl: "https://www.skills.sh/mattpocock/skills/triage",
      owner: "mattpocock",
      repository: "skills",
      skillName: "triage",
      repositoryUrl: "https://github.com/mattpocock/skills",
    });
    const wrapper = mount(AddSkillDialog, { props: { open: true, groups: [makeGroup()] } });

    await wrapper.find('input[type="url"]').setValue("https://www.skills.sh/mattpocock/skills/triage");
    await vi.waitFor(() => expect(backend.previewSkillUrl).toHaveBeenCalled());
    await flushPromises();
    await wrapper.find('input[type="text"]').setValue("Issue Triage");
    await wrapper.find("textarea").setValue("Custom description");

    expect((wrapper.find('input[type="text"]').element as HTMLInputElement).value).toBe("Issue Triage");
    expect((wrapper.find("textarea").element as HTMLTextAreaElement).value).toBe("Custom description");
  });

  it("warns about an existing canonical Skill and blocks duplicate submission", async () => {
    vi.mocked(backend.previewSkillUrl).mockResolvedValue({
      canonicalUrl: "https://www.skills.sh/mattpocock/skills/triage",
      owner: "mattpocock",
      repository: "skills",
      skillName: "triage",
      repositoryUrl: "https://github.com/mattpocock/skills",
    });
    const wrapper = mount(AddSkillDialog, {
      props: {
        open: true,
        groups: [makeGroup()],
        skills: [
          makeSkill({ displayName: "Existing Triage", skillsUrl: "https://www.skills.sh/mattpocock/skills/triage" }),
        ],
      },
    });

    await wrapper.find('input[type="url"]').setValue("https://www.skills.sh/mattpocock/skills/triage");
    await vi.waitFor(() => expect(backend.previewSkillUrl).toHaveBeenCalled());
    await flushPromises();

    expect(wrapper.get(".add-skill-dialog__duplicate").text()).toContain("Existing Triage");
    expect(wrapper.get('button[type="submit"]').attributes("disabled")).toBeDefined();
    await wrapper.find("form").trigger("submit");
    expect(wrapper.emitted("submit")).toBeUndefined();
  });

  it("shows a validation error and disables submit for a rejected URL", async () => {
    vi.mocked(backend.previewSkillUrl).mockRejectedValue(
      new Error("expected https://www.skills.sh/<owner>/<repository>/<skill-name>"),
    );

    const wrapper = mount(AddSkillDialog, {
      props: { open: true, groups: [makeGroup()] },
    });

    await wrapper.find('input[type="url"]').setValue("https://example.com/not-a-skill");
    await wrapper.find('input[type="url"]').trigger("input");
    await vi.waitFor(() => expect(backend.previewSkillUrl).toHaveBeenCalled());
    await flushPromises();

    expect(wrapper.find(".add-skill-dialog__error").exists()).toBe(true);
    const submitButton = wrapper.find('button[type="submit"]');
    expect(submitButton.attributes("disabled")).toBeDefined();
  });

  it("emits submit with the derived group and trimmed fields", async () => {
    vi.mocked(backend.previewSkillUrl).mockResolvedValue({
      canonicalUrl: "https://www.skills.sh/mattpocock/skills/triage",
      owner: "mattpocock",
      repository: "skills",
      skillName: "triage",
      repositoryUrl: "https://github.com/mattpocock/skills",
    });

    const wrapper = mount(AddSkillDialog, {
      props: { open: true, groups: [makeGroup({ id: "testing" })] },
    });

    await wrapper.find('input[type="url"]').setValue("https://www.skills.sh/mattpocock/skills/triage");
    await wrapper.find('input[type="url"]').trigger("input");
    await vi.waitFor(() => expect(backend.previewSkillUrl).toHaveBeenCalled());
    await flushPromises();

    await wrapper.find("form").trigger("submit");

    const emitted = wrapper.emitted("submit");
    expect(emitted).toBeTruthy();
    expect(emitted![0][0]).toMatchObject({
      url: "https://www.skills.sh/mattpocock/skills/triage",
      displayName: "triage",
      description: "mattpocock/skills",
      groupId: "testing",
    });
  });

  describe("Skills.sh catalog table", () => {
    const remote = makeSkill({ id: "remote-a", displayName: "Remote Skill A", source: { kind: "remote" } });
    const remoteB = makeSkill({ id: "remote-b", displayName: "Remote Skill B", source: { kind: "remote" } });
    const local = makeSkill({ id: "local-a", displayName: "Local Skill", source: { kind: "local", path: "/tmp/x" } });

    it("lists only Skills sourced from Skills.sh, not ones discovered locally", () => {
      const wrapper = mount(AddSkillDialog, {
        props: { open: true, groups: [makeGroup()], skills: [remote, remoteB, local] },
      });

      const rows = wrapper.findAll(".add-skill-dialog__row:not(.add-skill-dialog__row--header)");
      expect(rows.map((r) => r.text())).toEqual([expect.stringContaining("Remote Skill A"), expect.stringContaining("Remote Skill B")]);
      expect(wrapper.text()).not.toContain("Local Skill");
    });

    it("shows the Skill's name as plain text and its full add URL as the clickable link", () => {
      const wrapper = mount(AddSkillDialog, { props: { open: true, groups: [makeGroup()], skills: [remote] } });

      const name = wrapper.get(".add-skill-dialog__skill-name");
      expect(name.text()).toBe("Remote Skill A");
      expect(name.element.tagName).not.toBe("A");

      const link = wrapper.get(".add-skill-dialog__skill-link");
      expect(link.element.tagName).toBe("A");
      expect(link.text()).toBe(remote.skillsUrl);
    });

    it("shows an empty state when no Skills have been added from Skills.sh", () => {
      const wrapper = mount(AddSkillDialog, { props: { open: true, groups: [makeGroup()], skills: [local] } });
      expect(wrapper.get(".add-skill-dialog__catalog-empty").text()).toBe("No Skills added from Skills.sh yet.");
    });

    it("asks for confirmation before deleting a single row, and does nothing if declined", async () => {
      const confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(false);
      const wrapper = mount(AddSkillDialog, { props: { open: true, groups: [makeGroup()], skills: [remote] } });

      await wrapper.get(".add-skill-dialog__row-delete").trigger("click");
      expect(confirmSpy).toHaveBeenCalled();
      expect(wrapper.emitted("deleteSkills")).toBeUndefined();

      confirmSpy.mockReturnValue(true);
      await wrapper.get(".add-skill-dialog__row-delete").trigger("click");
      expect(wrapper.emitted("deleteSkills")).toEqual([[["remote-a"]]]);

      confirmSpy.mockRestore();
    });

    it("selects all Skills, then bulk-deletes the selection after confirming", async () => {
      vi.spyOn(window, "confirm").mockReturnValue(true);
      const wrapper = mount(AddSkillDialog, {
        props: { open: true, groups: [makeGroup()], skills: [remote, remoteB] },
      });

      expect(wrapper.find(".add-skill-dialog__bulk-delete").exists()).toBe(false);

      await wrapper.get('[aria-label="Select all Skills from Skills.sh"]').setValue(true);
      const headerRow = wrapper.get(".add-skill-dialog__row--header");
      const bulkDelete = headerRow.get(".add-skill-dialog__bulk-delete");
      expect(bulkDelete.attributes("aria-label")).toBe("Delete 2 selected Skills");
      expect(bulkDelete.find("svg").exists()).toBe(true);
      expect(bulkDelete.text()).toBe("");

      await bulkDelete.trigger("click");
      expect(wrapper.emitted("deleteSkills")).toEqual([[["remote-a", "remote-b"]]]);

      vi.restoreAllMocks();
    });

    it("emits edit with the Skill's id from its row's edit button", async () => {
      const wrapper = mount(AddSkillDialog, { props: { open: true, groups: [makeGroup()], skills: [remote] } });
      await wrapper.get(".add-skill-dialog__row-edit").trigger("click");
      expect(wrapper.emitted("edit")).toEqual([["remote-a"]]);
    });

    it("opens a Skill's Skills.sh URL through the plugin instead of navigating the webview", async () => {
      const wrapper = mount(AddSkillDialog, { props: { open: true, groups: [makeGroup()], skills: [remote] } });
      await wrapper.get(".add-skill-dialog__skill-link").trigger("click");
      expect(vi.mocked(openUrl)).toHaveBeenCalledWith(remote.skillsUrl);
    });

    it("disables row and bulk controls while a delete is in flight", () => {
      const wrapper = mount(AddSkillDialog, {
        props: { open: true, groups: [makeGroup()], skills: [remote], deleting: true },
      });
      expect(wrapper.get(".add-skill-dialog__row-delete").attributes("disabled")).toBeDefined();
    });

    it("shows a delete error when the backend rejects it", () => {
      const wrapper = mount(AddSkillDialog, {
        props: { open: true, groups: [makeGroup()], skills: [remote], deleteError: "Could not delete Skill" },
      });
      expect(wrapper.get(".add-skill-dialog__catalog .add-skill-dialog__error").text()).toBe("Could not delete Skill");
    });
  });
});
