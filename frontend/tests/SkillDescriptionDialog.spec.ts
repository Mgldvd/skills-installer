import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import SkillDescriptionDialog from "../src/components/SkillDescriptionDialog/SkillDescriptionDialog.vue";
import { makeSkill } from "./fixtures";

describe("SkillDescriptionDialog", () => {
  it("shows the complete description and skills.sh link", () => {
    const skill = makeSkill({
      description: "A complete Skill description.",
      skillsUrl: "https://www.skills.sh/owner/repo/skill",
    });
    const wrapper = mount(SkillDescriptionDialog, { props: { open: false, skill } });

    expect(wrapper.text()).toContain("A complete Skill description.");
    expect(wrapper.find("a").attributes("href")).toBe(skill.skillsUrl);
    expect(wrapper.find("a").attributes("rel")).toContain("noopener");
  });

  it("does not render a link for a local skill without a skills.sh page", () => {
    const wrapper = mount(SkillDescriptionDialog, {
      props: { open: false, skill: makeSkill({ local: true, skillsUrl: "" }) },
    });

    expect(wrapper.find("a").exists()).toBe(false);
    expect(wrapper.text()).toContain("Local Skill");
  });
});
