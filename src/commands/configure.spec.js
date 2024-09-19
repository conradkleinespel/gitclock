const { configure } = require("./configure");

describe("configure function", () => {
  test.each(["+0400", "+0200", "Europe/Paris"])(
    "sets timezone when it is in options",
    (timezone) => {
      const config = {
        setTimezone: jest.fn(),
        getFilePath: jest.fn().mockReturnValueOnce("/path/to/config"),
      };
      const options = { allowPushOutsideTimeslot: null, timezone };
      configure(config, options);
      expect(config.setTimezone).toHaveBeenCalledWith(timezone);
    },
  );

  test.each([true, false])(
    "sets allow_push_outside_timeslot when it is in options",
    (allowPushOutsideTimeslot) => {
      const config = {
        setAllowPushOutsideTimeslot: jest.fn(),
        getFilePath: jest.fn().mockReturnValueOnce("/path/to/config"),
      };
      const options = { allowPushOutsideTimeslot, timezone: null };
      configure(config, options);
      expect(config.setAllowPushOutsideTimeslot).toHaveBeenCalledWith(
        allowPushOutsideTimeslot,
      );
    },
  );

  test("does not set allow_push_outside_timeslot when it is not in options", () => {
    const config = {
      set: jest.fn(),
      getFilePath: jest.fn().mockReturnValueOnce("/path/to/config"),
    };
    const options = {};
    configure(config, options);
    expect(config.set).not.toHaveBeenCalled();
  });
});
