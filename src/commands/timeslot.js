const console = require("node:console");

function timeslot(options, config) {
  if (options.add && options.list) {
    console.error("Error: Options --add and --list are incompatible.");
    return 1;
  }

  if (options.add) {
    if (!options.days || !options.start || !options.end) {
      console.error("Error: Need --days, --start and --end.");
      return 1;
    }
    return timeslotAdd(options.days, options.start, options.end, config);
  }
  if (options.list) {
    return timeslotList(config);
  }

  console.error("Error: Need one of --add or --list.");
  return 1;
}

function timeslotAdd(days, start, end, config) {
  try {
    config.addTimeslot(days, start, end);
  } catch (err) {
    console.error(`Error: ${err.message}`);
    return 1;
  }

  console.log("Timeslot added.");

  console.log("");
  console.log("To remove a timeslot, edit:");
  console.log(`  ${config.getFilePath()}`);
  return 0;
}

function timeslotList(config) {
  const timeslots = config.getTimeslots();
  if (!timeslots.length) {
    console.log("No timeslots.");
    return 0;
  }

  console.log("Current timeslots:");
  for (let timeslot of timeslots) {
    console.log(timeslot.toString());
  }
  console.log("");
  console.log("To remove a timeslot, edit:");
  console.log(`  ${config.getFilePath()}`);

  return 0;
}

module.exports = {
  timeslot,
};
