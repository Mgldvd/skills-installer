/** Derives a Project's default name from a git remote URL — e.g.
 * "https://github.com/owner/repo.git" or the SSH-style
 * "git@github.com:owner/repo.git" both become "repo". Returns "" when
 * nothing usable can be extracted. */
export function repoNameFromGitUrl(raw: string): string {
  const trimmed = raw.trim().replace(/\/+$/, "");
  if (!trimmed) return "";
  const withoutGitSuffix = trimmed.replace(/\.git$/i, "");
  return (
    withoutGitSuffix
      .split(/[/:]/)
      .filter(Boolean)
      .pop() ?? ""
  );
}

export type GitHost = "github" | "gitlab";

/** Which of the two hosts a saved git URL points at, so the row can show
 * that host's icon — `null` for anything else (self-hosted GitLab/Gitea,
 * the SSH shorthand, or an unparseable value), which just means no icon. */
export function gitHostFromUrl(raw: string): GitHost | null {
  let host: string;
  try {
    host = new URL(raw).hostname.toLowerCase();
  } catch {
    return null;
  }
  if (host === "github.com" || host.endsWith(".github.com")) return "github";
  if (host === "gitlab.com" || host.endsWith(".gitlab.com")) return "gitlab";
  return null;
}
