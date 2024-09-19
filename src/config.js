const { Timeslot } = require("./timeslot");
const toml = require("@iarna/toml");
const Conf = require("conf");
const { DateTime } = require("luxon");

class Config {
  constructor(inner) {
    this.inner = inner;
  }

  static createFromConf() {
    return new this(
      new Conf({
        projectName: "gitclock",
        projectSuffix: "",
        fileExtension: "toml",
        serialize: toml.stringify,
        deserialize: toml.parse,
      }),
    );
  }

  get(key, defaultValue) {
    return this.inner.get(key, defaultValue);
  }

  set(key, value) {
    this.inner.set(key, value);
  }

  getTimeslots() {
    let timeslots = this.get("timeslots", []);

    return timeslots.map(
      ({ days, start, end }) =>
        new Timeslot(days, start, end, this.getTimezone()),
    );
  }

  addTimeslot(days, start, end) {
    new Timeslot(days, start, end, this.getTimezone());
    this.set("timeslots", [...this.get("timeslots", []), { days, start, end }]);
  }

  getAllowPushOutsideTimeslot() {
    return this.get("allow_push_outside_timeslot", false);
  }

  setAllowPushOutsideTimeslot(value) {
    this.set("allow_push_outside_timeslot", value);
  }

  getTimezone() {
    return this.get("timezone", DateTime.now().zoneName);
  }

  setTimezone(value) {
    if (!value || !DateTime.now().setZone(value).isValid) {
      throw new Error(
        `Timezone is invalid, expected something like Europe/Paris, but got ${value}.`,
      );
    }
    this.set("timezone", value);
  }

  getFilePath() {
    return this.inner.path;
  }

  checkConfig() {
    const timeslots = this.get("timeslots", []);
    if (!Array.isArray(timeslots)) {
      throw new Error("Timeslots must be a list");
    }

    for (let timeslot of timeslots) {
      new Timeslot(
        timeslot.days,
        timeslot.start,
        timeslot.end,
        this.getTimezone(),
      );
    }

    if (typeof this.get("allow_push_outside_timeslot", false) !== "boolean") {
      throw new Error("Allow push outside timeslot must be boolean");
    }

    const timezone = this.get("timezone");
    if (
      timezone != null &&
      (typeof timezone !== "string" ||
        !DateTime.now().setZone(timezone).isValid)
    ) {
      throw new Error(`Timezone must be string, eg +0200 or Europe/Paris`);
    }
  }
}

module.exports = {
  Config,
};
