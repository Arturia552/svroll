import { FieldTypeEnum, JsonStruct } from "@/types/mqttConfig";

export const convertToJsonStruct = (
  data: object,
  jsonStructArray: JsonStruct[]
): JsonStruct[] => {
  for (const [key, value] of Object.entries(data)) {
    if(jsonStructArray.filter(item=> item.fieldName === key)?.length !== 0) continue;
    const jsonStruct: JsonStruct = {
      fieldName: key,
      fieldType: getFieldType(value),
      possibleValues: getFieldType(value) === FieldTypeEnum.Object ? "" : value,
      children: [],
    };

    if (jsonStruct.fieldName === "timestamp") {
      const dateTimeRegex = /^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d{3}$/;
      if (typeof value === "string" && dateTimeRegex.test(value)) {
        jsonStruct.fieldType = FieldTypeEnum.DateTime;
      }
    }

    if (jsonStruct.fieldType === FieldTypeEnum.Object) {
      jsonStruct.children = convertToJsonStruct(value, []);
    }

    jsonStructArray.push(jsonStruct);
  }

  return jsonStructArray;
};


const getFieldType = (value: any): FieldTypeEnum => {
  switch (typeof value) {
    case "string":
      return FieldTypeEnum.String;
    case "number":
      return Number.isInteger(value)
        ? FieldTypeEnum.Integer
        : FieldTypeEnum.Float;
    case "boolean":
      return FieldTypeEnum.Boolean;
    case "object":
      if (value === null) {
        return FieldTypeEnum.Null;
      } else if (Array.isArray(value)) {
        return FieldTypeEnum.Array;
      } else {
        return FieldTypeEnum.Object;
      }
    default:
      return FieldTypeEnum.Unknown;
  }
};

export const isJsonValueNull = (
  jsonObj: Record<string, any>,
  ignore: string[]
): boolean => {
  for (const key in jsonObj) {
    if (ignore.includes(key)) {
      continue;
    }
    if (jsonObj[key] === null) { 
      return true;
    }
    if (typeof jsonObj[key] === "string" && jsonObj[key].trim() === "") {
      return true;
    }
    if (typeof jsonObj[key] === "object" && jsonObj[key] !== null) {
      if (isJsonValueNull(jsonObj[key], ignore)) {
        return true;
      }
    }
  }
  return false;
};

export const getNestedValue = (obj: any, path: string) => {
  return path.split(".").reduce((o, p) => (o ? o[p] : undefined), obj);
};
