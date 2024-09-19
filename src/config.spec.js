const { Config } = require("./config");
const { Timeslot } = require("./timeslot");
const Conf = require("conf");
const { afterEach } = require("@jest/globals");
const { DateTime } = require("luxon");

jest.mock("conf");

describe("Config class", () => {
  afterEach(() => {
    jest.resetAllMocks();
    jest.clearAllMocks();
  });

  test("constructor creates config class from inner config", () => {
    const configDict = {};
    const configMockGlobal = new Config({
      get: (key, defaultValue) =>
        Object.prototype.hasOwnProperty.call(configDict, key)
          ? configDict[key]
          : defaultValue,
      set: (key, value) => (configDict[key] = value),
    });

    expect(configMockGlobal.get("foo")).toBeUndefined();
    expect(configMockGlobal.get("foo", "baz")).toEqual("baz");
    configMockGlobal.set("foo", "bar");
    expect(configMockGlobal.get("foo")).toEqual("bar");
  });

  test("should create the class instance from Conf", () => {
    Config.createFromConf();
    expect(Conf).toHaveBeenCalled();
  });

  test("getTimeslots() converts timeslots to instances of Timeslot class", () => {
    const configDict = {};
    const configMockGlobal = new Config({
      get: (key, defaultValue) =>
        Object.prototype.hasOwnProperty.call(configDict, key)
          ? configDict[key]
          : defaultValue,
      set: (key, value) => (configDict[key] = value),
    });

    expect(configMockGlobal.getTimeslots().length).toBe(0);
    configMockGlobal.addTimeslot("1-5", "0900", "1700");
    configMockGlobal.addTimeslot("6-6", "0900", "1200");
    const timeslots = configMockGlobal.getTimeslots();
    expect(timeslots.length).toBe(2);
    timeslots.forEach((timeslot) => expect(timeslot).toBeInstanceOf(Timeslot));
  });

  test("getFilePath() returns underlying path", () => {
    expect(new Config({ path: "/path/to/config" }).getFilePath()).toEqual(
      "/path/to/config",
    );
  });

  test("getAllowPushOutsideTimeslot() returns boolean", () => {
    const configMock = {
      get: jest.fn().mockReturnValueOnce(false),
      set: jest.fn(),
    };
    const config = new Config(configMock);
    expect(config.getAllowPushOutsideTimeslot()).toBe(false);
    expect(configMock.get).toHaveBeenCalledWith(
      "allow_push_outside_timeslot",
      false,
    );
    config.setAllowPushOutsideTimeslot(true);
    expect(configMock.set).toHaveBeenCalledWith(
      "allow_push_outside_timeslot",
      true,
    );
  });

  test("getTimezone() returns null when undefined or the timezone when defined", () => {
    const configMock = {
      get: jest.fn().mockReturnValueOnce(null),
      set: jest.fn(),
    };
    const config = new Config(configMock);
    expect(config.getTimezone()).toBe(null);
    expect(DateTime.now().zoneName.length).toBeTruthy();
    expect(configMock.get).toHaveBeenCalledWith(
      "timezone",
      DateTime.now().zoneName,
    );
    config.setTimezone("+0300");
    expect(configMock.set).toHaveBeenCalledWith("timezone", "+0300");
  });

  test("setTimezone() throws when called with invalid timezone", () => {
    const configMock = {
      get: jest.fn().mockReturnValueOnce(null),
      set: jest.fn(),
    };
    const config = new Config(configMock);
    expect(() => config.setTimezone("invalid")).toThrow();
  });

  test.each([undefined, null, "Europe/Paris"])(
    "checkConfig() is noop when configuration is valid",
    (timezone) => {
      const configMock = {
        get: jest
          .fn()
          .mockReturnValueOnce([{ days: "1-5", start: "0900", end: "1700" }])
          .mockReturnValueOnce("Europe/Paris")
          .mockReturnValueOnce(true)
          .mockReturnValueOnce(timezone),
      };
      const config = new Config(configMock);
      config.checkConfig();
      expect(configMock.get).toHaveBeenCalledWith("timeslots", []);
      expect(configMock.get).toHaveBeenCalledWith(
        "allow_push_outside_timeslot",
        false,
      );
    },
  );

  test("checkConfig() throws when timeslots configuration is invalid", () => {
    const configMock = {
      get: jest.fn().mockReturnValueOnce("invalid"),
    };
    const config = new Config(configMock);
    expect(() => config.checkConfig()).toThrow("Timeslots must be a list");
  });

  test.each(["invalid", "Europe/Paris"])(
    "checkConfig() throws when timezone configuration is invalid",
    (timezoneDefault) => {
      const configMock = {
        get: jest
          .fn()
          .mockReturnValueOnce([{ days: "1-5", start: "0900", end: "1700" }])
          .mockReturnValueOnce(timezoneDefault)
          .mockReturnValueOnce(true)
          .mockReturnValueOnce("invalid"),
      };
      const config = new Config(configMock);
      expect(() => config.checkConfig()).toThrow(
        "Timezone must be string, eg +0200 or Europe/Paris",
      );
    },
  );

  test("checkConfig() throws when push outside timeslot configuration is invalid", () => {
    const configMock = {
      get: jest
        .fn()
        .mockReturnValueOnce([{ days: "1-5", start: "0900", end: "1700" }])
        .mockReturnValueOnce("Europe/Paris")
        .mockReturnValueOnce("not boolean"),
    };
    const config = new Config(configMock);
    expect(() => config.checkConfig()).toThrow(
      "Allow push outside timeslot must be boolean",
    );
  });
});
