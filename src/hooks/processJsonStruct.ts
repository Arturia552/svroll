import { FieldTypeEnum, JsonStruct, PossibleValue } from "@/types/mqttConfig"

export const convertToJsonStruct = (
  data: object,
  jsonStructArray: JsonStruct[]
): JsonStruct[] => {
  for (const [key, value] of Object.entries(data)) {
    if(jsonStructArray.filter(item=> item.fieldName === key)?.length !== 0) {
      // 如果已经存在相同的字段名，则跳过，并赋予id
      const existingStruct = jsonStructArray.find(item => item.fieldName === key)
      if(existingStruct) {
        existingStruct.id = Math.floor(Math.random() * 1000000)
      }
      continue
    }

    const jsonStruct: JsonStruct = {
      id: Math.floor(Math.random() * 1000000),
      fieldName: key,
      fieldType: getFieldType(value),
      possibleValues: [{ value: value, probability:1 }] as PossibleValue[],
    }

    if (jsonStruct.fieldName === "timestamp") {
      const dateTimeRegex = /^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d{3}$/
      if (typeof value === "string" && dateTimeRegex.test(value)) {
        jsonStruct.fieldType = FieldTypeEnum.DateTime
      }
    }
    jsonStructArray.push(jsonStruct)
  }
  return jsonStructArray
}


const getFieldType = (value: any): FieldTypeEnum => {
  switch (typeof value) {
    case "string":
      return FieldTypeEnum.String
    case "number":
      return Number.isInteger(value)
        ? FieldTypeEnum.Integer
        : FieldTypeEnum.Float
    case "boolean":
      return FieldTypeEnum.Boolean
    case "object":
      if (value === null) {
        return FieldTypeEnum.Null
      } else if (Array.isArray(value)) {
        return FieldTypeEnum.Array
      } else {
        return FieldTypeEnum.String
      }
    default:
      return FieldTypeEnum.Unknown
  }
}

export const isJsonValueNull = (
  jsonObj: Record<string, any>,
  ignore: string[]
): boolean => {
  for (const key in jsonObj) {
    if (ignore.includes(key)) {
      continue
    }
    if (jsonObj[key] === null) { 
      return true
    }
    if (typeof jsonObj[key] === "string" && jsonObj[key].trim() === "") {
      return true
    }
    if (typeof jsonObj[key] === "object" && jsonObj[key] !== null) {
      if (isJsonValueNull(jsonObj[key], ignore)) {
        return true
      }
    }
  }
  return false
}

export const getNestedValue = (obj: any, path: string) => {
  return path.split(".").reduce((o, p) => (o ? o[p] : undefined), obj)
}

export const convertJsonStructToJson = (jsonStructArray: JsonStruct[]): object => {
  const result: any = {}
  jsonStructArray.forEach((item) => {
    if (item.fieldType === FieldTypeEnum.Array) {
      result[item.fieldName] = []
    } else if (item.possibleValues && item.possibleValues.length > 0) {
      const value = item.possibleValues[0].value
      
      // 根据字段类型转换值
      switch (item.fieldType) {
        case FieldTypeEnum.Integer:
          result[item.fieldName] = Number.isInteger(value) ? value : Math.floor(Number(value))
          break
        case FieldTypeEnum.Float:
          result[item.fieldName] = Number(value)
          break
        case FieldTypeEnum.Boolean:
          result[item.fieldName] = Boolean(value)
          break
        case FieldTypeEnum.String:
          result[item.fieldName] = String(value)
          break
        case FieldTypeEnum.Timestamp:
          result[item.fieldName] = Number(value)
          break
        case FieldTypeEnum.DateTime:
          result[item.fieldName] = String(value)
          break
        case FieldTypeEnum.Enum:
          result[item.fieldName] = value
          break
        default:
          result[item.fieldName] = value
          break
      }
    } else {
      // 没有可能值时，根据类型设置默认值
      switch (item.fieldType) {
        case FieldTypeEnum.Integer:
        case FieldTypeEnum.Float:
        case FieldTypeEnum.Timestamp:
          result[item.fieldName] = 0
          break
        case FieldTypeEnum.Boolean:
          result[item.fieldName] = false
          break
        case FieldTypeEnum.String:
        case FieldTypeEnum.DateTime:
          result[item.fieldName] = ""
          break
        default:
          result[item.fieldName] = null
          break
      }
    }
  })
  return result
}
