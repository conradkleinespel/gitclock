const console = require("node:console");

exports.configure = function (config, options) {
  if (options.allowPushOutsideTimeslot != null) {
    config.setAllowPushOutsideTimeslot(options.allowPushOutsideTimeslot);
    console.log(
      `Setting allow_push_outside_timeslot = ${options.allowPushOutsideTimeslot ? "true" : "false"}.`,
    );
  }

  if (options.timezone != null) {
    config.setTimezone(options.timezone);
    console.log(`Setting timezone = ${options.timezone}.`);
  }

  console.log("Done.");

  console.log("");
  console.log("To view your configuration, open:");
  console.log(`  ${config.getFilePath()}`);

  return 0;
};
