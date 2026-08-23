import { describe, expect, it } from "vitest";

import { gitHostFromUrl, repoNameFromGitUrl } from "../src/utils/gitUrl";

describe("repoNameFromGitUrl", () => {
  it("extracts the repo name from an HTTPS URL", () => {
    expect(repoNameFromGitUrl("https://github.com/Mgldvd/skills-installer")).toBe("skills-installer");
  });

  it("strips a trailing .git suffix", () => {
    expect(repoNameFromGitUrl("https://github.com/Mgldvd/skills-installer.git")).toBe("skills-installer");
  });

  it("strips a trailing slash", () => {
    expect(repoNameFromGitUrl("https://github.com/Mgldvd/skills-installer/")).toBe("skills-installer");
  });

  it("handles the SSH-style shorthand", () => {
    expect(repoNameFromGitUrl("git@github.com:Mgldvd/skills-installer.git")).toBe("skills-installer");
  });

  it("returns an empty string for blank or unusable input", () => {
    expect(repoNameFromGitUrl("")).toBe("");
    expect(repoNameFromGitUrl("   ")).toBe("");
  });
});

describe("gitHostFromUrl", () => {
  it("recognizes github.com", () => {
    expect(gitHostFromUrl("https://github.com/Mgldvd/skills-installer")).toBe("github");
  });

  it("recognizes gitlab.com", () => {
    expect(gitHostFromUrl("https://gitlab.com/me/repo")).toBe("gitlab");
  });

  it("recognizes a self-hosted subdomain of either host", () => {
    expect(gitHostFromUrl("https://code.github.com/me/repo")).toBe("github");
    expect(gitHostFromUrl("https://code.gitlab.com/me/repo")).toBe("gitlab");
  });

  it("returns null for any other host", () => {
    expect(gitHostFromUrl("https://bitbucket.org/me/repo")).toBeNull();
  });

  it("returns null for the SSH shorthand and unparseable input", () => {
    expect(gitHostFromUrl("git@github.com:me/repo.git")).toBeNull();
    expect(gitHostFromUrl("not a url")).toBeNull();
  });
});
