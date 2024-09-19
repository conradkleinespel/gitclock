const console = require("node:console");
const {
  getLogShaAndDates,
  cherryPick,
  resetHard,
  amendWithNewDate,
} = require("../git");
const { getNextCommitDate } = require("./commit");
const { DateTime } = require("luxon");

function chooseMinDateForNewCommit(
  existingAuthorDate,
  existingCommitDate,
  lastCommitDate,
) {
  const minDateFromLogEntry =
    existingAuthorDate > existingCommitDate
      ? existingAuthorDate
      : existingCommitDate;

  return lastCommitDate
    ? lastCommitDate > minDateFromLogEntry
      ? lastCommitDate
      : minDateFromLogEntry
    : minDateFromLogEntry;
}

async function amendCommit(logEntry, timeslots, lastCommitDate, timezone) {
  const minDate = chooseMinDateForNewCommit(
    logEntry.authorDate,
    logEntry.commitDate,
    lastCommitDate,
  );
  const currentDate = DateTime.now();
  const newCommitDate = getNextCommitDate(currentDate, minDate, timeslots);
  await amendWithNewDate(newCommitDate, timezone);
  return newCommitDate;
}

async function rewriteHistory(config) {
  const timeslots = config.getTimeslots();
  if (!timeslots.length) {
    console.log("No timeslots found. Please add timeslots.");
    return 1;
  }

  console.log(`Rewriting commit dates.`);

  const logEntries = await getLogShaAndDates();

  // Amend the first commit to bootstrap the process, since we can't easily reset to no commits at all
  await resetHard(logEntries[0].sha);
  let lastCommitDate = await amendCommit(
    logEntries[0],
    timeslots,
    null,
    config.getTimezone(),
  );
  for (let logEntry of logEntries.slice(1)) {
    await cherryPick(logEntry.sha);
    lastCommitDate = await amendCommit(
      logEntry,
      timeslots,
      lastCommitDate,
      config.getTimezone(),
    );
  }

  return 0;
}

module.exports = {
  rewriteHistory,
  chooseMinDateForNewCommit,
};
