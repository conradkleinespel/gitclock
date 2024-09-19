const { DateTime } = require("luxon");

class Timeslot {
  constructor(dayRange, startTime, endTime, timezone) {
    if (
      typeof dayRange !== "string" ||
      typeof startTime !== "string" ||
      typeof endTime !== "string"
    ) {
      throw new Error(
        `Invalid timeslot [days=${dayRange}, start=${startTime}, end=${endTime}].`,
      );
    }

    this.dayRange = dayRange.split("-").map(Number);
    if (!/^[1-7]{1}-[1-7]{1}$/.test(dayRange)) {
      throw new Error("Invalid day range format. It should be like 1-7");
    }
    if (this.dayRange[0] > this.dayRange[1]) {
      throw new Error(
        "Invalid day range, first day should be lower or equal to second day",
      );
    }

    this.startTime = {
      hour: Number(startTime.slice(0, 2)),
      minute: Number(startTime.slice(2, 4)),
    };
    if (!/^[0-2][0-9][0-5][0-9]$/.test(startTime) || this.startTime.hour > 23) {
      throw new Error(
        "Invalid start time format, it should be like 0900 or 1530",
      );
    }

    this.endTime = {
      hour: Number(endTime.slice(0, 2)),
      minute: Number(endTime.slice(2, 4)),
    };
    if (!/^[0-2][0-9][0-5][0-9]$/.test(endTime) || this.endTime.hour > 23) {
      throw new Error(
        "Invalid end time format, it should be like 0900 or 1530",
      );
    }

    if (!timezone || !DateTime.now().setZone(timezone).isValid) {
      throw new Error("Timezone must be string, eg +0200 or Europe/Paris");
    }
    this.timezone = timezone;
  }

  isDateWithin(date) {
    date = date.setZone(this.timezone);

    const day = date.weekday;
    const hour = date.hour;
    const minute = date.minute;

    if (day < this.dayRange[0] || day > this.dayRange[1]) {
      return false;
    }
    if (
      hour < this.startTime.hour ||
      (hour === this.startTime.hour && minute < this.startTime.minute)
    ) {
      return false;
    }
    if (
      hour > this.endTime.hour ||
      (hour === this.endTime.hour && minute > this.endTime.minute)
    ) {
      return false;
    }
    return true;
  }

  nextSuitableDate(minDate) {
    while (!this.isDateWithin(minDate)) {
      minDate = minDate.plus({ minutes: 1 });
    }
    return minDate;
  }

  toString() {
    const days = [
      "Monday",
      "Tuesday",
      "Wednesday",
      "Thursday",
      "Friday",
      "Saturday",
      "Sunday",
    ];

    const formattedStartTime = `${String(this.startTime.hour).padStart(2, "0")}:${String(this.startTime.minute).padStart(2, "0")}`;
    const formattedEndTime = `${String(this.endTime.hour).padStart(2, "0")}:${String(this.endTime.minute).padStart(2, "0")}`;

    let dayRangeStatement = days[this.dayRange[0] - 1];
    if (this.dayRange[0] !== this.dayRange[1]) {
      dayRangeStatement += ` to ${days[this.dayRange[1] - 1]}`;
    }

    return `${dayRangeStatement}, between ${formattedStartTime} and ${formattedEndTime} in timezone ${this.timezone}`;
  }
}

module.exports = {
  Timeslot,
};
