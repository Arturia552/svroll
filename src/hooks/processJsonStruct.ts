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

    const detectedType = getFieldType(value)
    const jsonStruct: JsonStruct = {
      id: Math.floor(Math.random() * 1000000),
      fieldName: key,
      fieldType: detectedType,
      possibleValues: detectedType === FieldTypeEnum.Object || detectedType === FieldTypeEnum.Array
        ? []
        : ([{ value: value as any, probability: 100 }] as PossibleValue[]),
      children: detectedType === FieldTypeEnum.Object ? [] : undefined,
    }

    if (jsonStruct.fieldName === "timestamp") {
      const dateTimeRegex = /^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d{3}$/
      if (typeof value === "string" && dateTimeRegex.test(value)) {
        jsonStruct.fieldType = FieldTypeEnum.DateTime
      }
    }
    
    // 如果是对象类型，递归处理子属性
    if (detectedType === FieldTypeEnum.Object && value && typeof value === 'object' && !Array.isArray(value)) {
      jsonStruct.children = convertToJsonStruct(value as Record<string, any>, jsonStruct.children || [])
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
        return FieldTypeEnum.Object
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

  const assignValue = (resObj: any, item: JsonStruct) => {
    if (!item.fieldName) return

    // Arrays currently default to empty array
    if (item.fieldType === FieldTypeEnum.Array) {
      resObj[item.fieldName] = []
      return
    }

    // Object: recursively convert children
    if (item.fieldType === FieldTypeEnum.Object) {
      const childObj: any = {}
      ;(item.children || []).forEach(child => assignValue(childObj, child))
      resObj[item.fieldName] = childObj
      return
    }

    if (item.possibleValues && item.possibleValues.length > 0) {
      const value = item.possibleValues[0].value

      switch (item.fieldType) {
        case FieldTypeEnum.Integer:
          resObj[item.fieldName] = Number.isInteger(value) ? value : Math.floor(Number(value))
          break
        case FieldTypeEnum.Float:
          resObj[item.fieldName] = Number(value)
          break
        case FieldTypeEnum.Boolean:
          resObj[item.fieldName] = Boolean(value)
          break
        case FieldTypeEnum.String:
          resObj[item.fieldName] = String(value)
          break
        case FieldTypeEnum.Timestamp:
          resObj[item.fieldName] = Number(value)
          break
        case FieldTypeEnum.DateTime:
          resObj[item.fieldName] = String(value)
          break
        case FieldTypeEnum.Enum:
          resObj[item.fieldName] = value
          break
        default:
          resObj[item.fieldName] = value
          break
      }
      return
    }

    // 默认值
    switch (item.fieldType) {
      case FieldTypeEnum.Integer:
      case FieldTypeEnum.Float:
      case FieldTypeEnum.Timestamp:
        resObj[item.fieldName] = 0
        break
      case FieldTypeEnum.Boolean:
        resObj[item.fieldName] = false
        break
      case FieldTypeEnum.String:
      case FieldTypeEnum.DateTime:
        resObj[item.fieldName] = ""
        break
      default:
        resObj[item.fieldName] = null
        break
    }
  }

  jsonStructArray.forEach(item => assignValue(result, item))
  return result
}

// 智能合并现有fieldStruct与新数据
export const mergeFieldStructWithData = (existingStruct: JsonStruct[], data: any): JsonStruct[] => {
  const result = [...existingStruct]
  
  // 递归处理函数
  const processLevel = (structArray: JsonStruct[], dataObj: any) => {
    // 为数据中的每个字段检查是否在结构中存在
    for (const [key, value] of Object.entries(dataObj)) {
      const existingField = structArray.find(field => field.fieldName === key)
      
      if (existingField) {
        // 字段已存在，更新其可能的值（但保留配置）
        if (existingField.fieldType !== FieldTypeEnum.Object && existingField.fieldType !== FieldTypeEnum.Array) {
          // 只有当原有possibleValues为空或者值发生变化时才更新
          if (!existingField.possibleValues || existingField.possibleValues.length === 0) {
            existingField.possibleValues = [{ value: value, probability: 100 }]
          } else {
            // 检查新值是否已在possibleValues中
            const hasValue = existingField.possibleValues.some(pv => pv.value === value)
            if (!hasValue) {
              // 更新第一个可能值为新值（保持配置不变）
              existingField.possibleValues[0].value = value
            }
          }
        } else if (existingField.fieldType === FieldTypeEnum.Object && existingField.children) {
          // 对象类型，递归处理子字段
          processLevel(existingField.children, value)
        }
      } else {
        // 字段不存在，添加新字段
        const detectedType = getFieldType(value)
        
        const newField: JsonStruct = {
          id: Math.floor(Math.random() * 1000000),
          fieldName: key,
          fieldType: detectedType,
          possibleValues: detectedType === FieldTypeEnum.Object || detectedType === FieldTypeEnum.Array
            ? []
            : [{ value: value, probability: 100 }],
          children: detectedType === FieldTypeEnum.Object ? [] : undefined,
        }
        
        // 如果是对象，递归处理子字段
        if (detectedType === FieldTypeEnum.Object && value && typeof value === 'object' && !Array.isArray(value)) {
          newField.children = []
          processLevel(newField.children, value)
        }
        
        structArray.push(newField)
      }
    }
  }
  
  processLevel(result, data)
  return result
}

// 同步编辑器数据到fieldStruct
export const syncFieldStructFromEditorData = (
  sendData: string, 
  existingFieldStruct: JsonStruct[]
): JsonStruct[] => {
  try {
    // 解析编辑器中的JSON数据
    const parsedData = JSON.parse(sendData || '{}')
    
    // 如果编辑器中有有效数据，则更新fieldStruct
    if (parsedData && Object.keys(parsedData).length > 0) {
      if (existingFieldStruct.length === 0) {
        // 如果没有现有结构，直接生成新的
        return convertToJsonStruct(parsedData, [])
      } else {
        // 如果有现有结构，智能合并
        return mergeFieldStructWithData(existingFieldStruct, parsedData)
      }
    }
    
    return existingFieldStruct
  } catch (error) {
    console.warn('同步编辑器数据到fieldStruct失败:', error)
    return existingFieldStruct
  }
}
