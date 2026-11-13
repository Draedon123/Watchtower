abstract class DataType {
  public abstract readonly stringName: string;
}

class F32Value extends DataType {
  public static readonly stringName = "f32";
  public readonly stringName = "f32";
  public value: number;

  constructor(value: number) {
    super();

    this.value = value;
  }

  public static fromString(string: string): F32Value {
    return new F32Value(parseFloat(string));
  }
}

class Vec2U64Value extends DataType {
  public static readonly stringName = "vec2u64";
  public readonly stringName = "vec2u64";
  public value: [number, number];

  constructor(a: number, b: number) {
    super();

    this.value = [a, b];
  }

  public static fromString(string: string): Vec2U64Value {
    const value = string.split(",").map((value) => parseInt(value));

    return new Vec2U64Value(value[0], value[1]);
  }
}

class UnknownValue extends DataType {
  public static readonly stringName = "unknown";
  public readonly stringName = "unknown";
  public value: string;

  constructor(value: string) {
    super();

    this.value = value;
  }

  public static fromString(string: string): UnknownValue {
    return new UnknownValue(string);
  }
}

function stringToValue(type: string, string: string): DataType {
  switch (type) {
    case F32Value.stringName: {
      return F32Value.fromString(string);
    }

    case Vec2U64Value.stringName: {
      return Vec2U64Value.fromString(string);
    }

    default: {
      return UnknownValue.fromString(string);
    }
  }
}

export { DataType, F32Value, Vec2U64Value, UnknownValue, stringToValue };
