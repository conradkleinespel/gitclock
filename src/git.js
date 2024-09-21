const { SpawnError, spawnAsync } = require("./spawnAsync");
const { DateTime } = require("luxon");
const process = require("node:process");

function formatJsDateToGitDate(date, timezone) {
  return date.setZone(timezone).toFormat("X ZZZ");
}

async function gitCommit(date, timezone, args) {
  const gitDate = formatJsDateToGitDate(date, timezone);
  return await runCommandWithArgs("commit", ["--date", gitDate, ...args], {
    GIT_AUTHOR_DATE: gitDate,
    GIT_COMMITTER_DATE: gitDate,
  });
}

async function getLastCommitDate() {
  let result;
  try {
    result = await spawnAsync("git", ["log", "-1", "--format=%cI"]);
  } catch (err) {
    if (err instanceof SpawnError) {
      // No commits
      return DateTime.now();
    }
    throw err;
  }
  return DateTime.fromISO(result.stdout.trim());
}

async function getTrackingRemoteAndBranch() {
  let result;
  try {
    result = await spawnAsync("git", ["rev-parse", "--abbrev-ref", "@{push}"]);
  } catch (err) {
    if (err instanceof SpawnError) {
      return { remote: null, branch: null, error: err.stderr };
    }
    throw err;
  }
  let [remote, branch] = result.stdout.trim().split("/");
  return { remote, branch, error: null };
}

async function getFirstPastCommitHash() {
  let result;
  try {
    result = await spawnAsync("git", [
      "log",
      '--until="now"',
      "--pretty=format:%H",
      "-1",
    ]);
  } catch (err) {
    if (err instanceof SpawnError) {
      return { commitHash: null, error: err.stderr };
    }
    throw err;
  }
  return { commitHash: result.stdout.trim(), error: null };
}

async function getPushObjectDate(objectName) {
  const result = await spawnAsync("git", [
    "show",
    "-s",
    "--format=%cI",
    objectName,
  ]);
  return DateTime.fromISO(result.stdout.trim());
}

class LogEntry {
  constructor(sha, authorDate, commitDate) {
    this.sha = sha;
    this.authorDate = authorDate;
    this.commitDate = commitDate;
    Object.freeze(this);
  }
}

async function getLogShaAndDates() {
  const result = await spawnAsync("git", [
    "log",
    "--pretty=format:%H %aI %cI",
    "--reverse",
  ]);

  return result.stdout
    .trim()
    .split("\n")
    .map((line) => line.trim())
    .map((line) => {
      const [sha, authorDate, commitDate] = line.split(" ");
      return new LogEntry(
        sha,
        DateTime.fromISO(authorDate),
        DateTime.fromISO(commitDate),
      );
    });
}

async function cherryPick(sha) {
  return await runCommandWithArgs("cherry-pick", [sha]);
}

async function resetHard(sha) {
  return await runCommandWithArgs("reset", ["--hard", sha]);
}

async function amendWithNewDate(newDate, timezone) {
  const gitDate = formatJsDateToGitDate(newDate, timezone);
  return await runCommandWithArgs(
    "commit",
    ["--amend", "--no-edit", "--date", gitDate],
    {
      GIT_AUTHOR_DATE: gitDate,
      GIT_COMMITTER_DATE: gitDate,
    },
  );
}

async function gitPush(args) {
  return await runCommandWithArgs("push", args);
}

async function gitRebase(args) {
  return await runCommandWithArgs("rebase", args);
}

async function runCommandWithArgs(command, args, env = {}) {
  try {
    await spawnAsync("git", [command, ...args], {
      stdio: "inherit",
      env: {
        ...process.env,
        ...env,
        GIT_CLOCK: "1",
      },
    });
  } catch (err) {
    if (err instanceof SpawnError) {
      return err.code;
    }
    throw err;
  }

  return 0;
}

module.exports = {
  gitCommit,
  getLastCommitDate,
  getTrackingRemoteAndBranch,
  getFirstPastCommitHash,
  getPushObjectDate,
  LogEntry,
  getLogShaAndDates,
  cherryPick,
  resetHard,
  amendWithNewDate,
  gitPush,
  gitRebase,
};
