<template>
  <div v-loading="loading" class="data-model config-component">
    <div class="header">
      <div class="actions">
        <el-tooltip content="刷新数据结构" placement="top">
          <el-button type="primary" size="small" circle @click="refreshStructure">
            <el-icon>
              <refresh />
            </el-icon>
          </el-button>
        </el-tooltip>
        <el-tooltip content="添加字段" placement="top">
          <el-button type="success" size="small" circle @click="showAddFieldDialog">
            <el-icon>
              <plus />
            </el-icon>
          </el-button>
        </el-tooltip>
        <el-tooltip content="导入模板" placement="top">
          <el-button type="info" size="small" circle @click="importTemplate">
            <el-icon>
              <download />
            </el-icon>
          </el-button>
        </el-tooltip>
      </div>
    </div>

    <el-divider />

    <div class="table-container">
      <el-table :data="config.fieldStruct"
                style="width: 100%"
                :height="ModelTableHeight"
                size="small"
                row-key="fieldName"
                :tree-props="{ children: 'children' }"
                border
                stripe
                highlight-current-row
                :row-style="{ height: '32px' }"
                :header-row-style="{ height: '36px' }">
        <el-table-column prop="fieldName" label="键值" min-width="20%" />

        <el-table-column prop="fieldType" label="数据类型" min-width="20%">
          <template #default="scope">
            <div class="cell-content">
              {{ getFieldTypeName(scope.row.fieldType) }}
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="minValue" label="最小值" min-width="15%">
          <template #default="scope">
            <div class="cell-content">
              {{ isContainerType(scope.row.fieldType) ? "--" : (scope.row.minValue ?? "--") }}
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="maxValue" label="最大值" min-width="15%">
          <template #default="scope">
            <div class="cell-content">
              {{ isContainerType(scope.row.fieldType) ? "--" : (scope.row.maxValue ?? "--") }}
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="possibleValues" label="有效值" min-width="20%">
          <template #default="scope">
            <div class="cell-content">
              <template v-if="scope.row.fieldType === FieldTypeEnum.Object">
                <el-tag type="info">
                  子字段：{{ scope.row.children?.length || 0 }}
                </el-tag>
              </template>
              <template v-else>
                <template v-if="scope.row.possibleValues && scope.row.possibleValues.length > 0">
                  <el-tag v-for="(item, index) in scope.row.possibleValues" :key="index" class="possible-value-tag">
                    <span>{{ item.value }}</span>
                    <span v-if="scope.row.fieldType === FieldTypeEnum.Enum">
                      ({{ item.probability }}%)
                    </span>
                  </el-tag>
                </template>
                <span v-else class="no-value">--</span>
              </template>
            </div>
          </template>
        </el-table-column>

        <el-table-column align="center" label="操作" width="100">
          <template #default="scope">
            <el-button type="primary" size="small" @click="showEditFieldDialog(scope.row)">
              编辑
            </el-button>
          </template>
        </el-table-column>

        <template #empty>
          <div class="empty-data">
            <el-empty description="暂无数据结构" :image-size="120">
              <template #extra>
                <el-button type="primary" @click="showAddFieldDialog">
                  添加字段
                </el-button>
              </template>
            </el-empty>
          </div>
        </template>
      </el-table>
    </div>

    <el-dialog v-model="addFieldDialogVisible"
               title="添加字段"
               width="500px"
               :close-on-click-modal="false"
               :append-to-body="true">
      <el-form ref="fieldFormRef" :model="newField" label-width="100px" :rules="fieldRules">
        <el-form-item label="添加到" prop="parentField">
          <el-select v-model="selectedParentFieldId" placeholder="请选择添加位置（空为根级）" style="width: 100%" clearable>
            <el-option value="" label="根级字段" />
            <el-option v-for="field in objectFields" :key="field.id" :label="field.label" :value="field.id" />
          </el-select>
        </el-form-item>
        <el-form-item label="字段名称" prop="fieldName">
          <el-input v-model="newField.fieldName" placeholder="请输入字段名称" />
        </el-form-item>
        <el-form-item label="字段类型" prop="fieldType">
          <el-select v-model="newField.fieldType" placeholder="请选择字段类型" style="width: 100%">
            <el-option v-for="item in fieldTypeOptions" :key="item.value" :label="item.label" :value="item.value" />
          </el-select>
        </el-form-item>
        <template v-if="!isContainerType(newField.fieldType)">
          <el-form-item v-if="newField.fieldType" label="有效值">
            <div v-if="newField.fieldType === FieldTypeEnum.Enum">
              <div class="possible-values-table">
                <el-table :data="possibleValuesArray" border size="default" max-height="200">
                  <el-table-column label="值" min-width="120">
                    <template #default="scope">
                      <el-input-number v-model="scope.row.value"
                                       :min="0"
                                       placeholder="值"
                                       size="small"
                                       style="width: 100%" />
                    </template>
                  </el-table-column>
                  <el-table-column label="概率(%)" min-width="120">
                    <template #default="scope">
                      <el-input-number v-model="scope.row.probability"
                                       :min="0"
                                       :max="100"
                                       placeholder="概率"
                                       size="small"
                                       style="width: 100%" />
                    </template>
                  </el-table-column>
                  <el-table-column label="操作" width="80" align="center">
                    <template #default="scope">
                      <el-button type="danger" circle size="small" @click="removePossibleValue(scope.$index)">
                        <el-icon>
                          <delete />
                        </el-icon>
                      </el-button>
                    </template>
                  </el-table-column>
                </el-table>
                <div class="table-actions">
                  <el-button type="primary" size="small" @click="addPossibleValue">
                    添加枚举值
                  </el-button>
                </div>
              </div>
            </div>
            <div v-else>
              <!-- 非枚举类型只有一个值 -->
              <el-input v-model="singleDefaultValue" placeholder="有效值" style="width: 100%" />
            </div>
          </el-form-item>
          <template v-if="isNumericType(newField.fieldType)">
            <el-form-item label="最小值">
              <el-input-number v-model="newField.minValue"
                               :precision="newField.fieldType === FieldTypeEnum.Float ? 2 : 0" />
            </el-form-item>
            <el-form-item label="最大值">
              <el-input-number v-model="newField.maxValue"
                               :precision="newField.fieldType === FieldTypeEnum.Float ? 2 : 0" />
            </el-form-item>
          </template>
        </template>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="addFieldDialogVisible = false">取消</el-button>
          <el-button type="primary" @click="confirmAddField">确认</el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 编辑字段对话框 -->
    <el-dialog v-model="editFieldDialogVisible"
               title="编辑字段"
               width="500px"
               :close-on-click-modal="false"
               :append-to-body="true">
      <el-form ref="editFieldFormRef" :model="currentEditField" label-width="80px" :rules="fieldRules">
        <el-form-item label="字段名称" prop="fieldName">
          <el-input v-model="currentEditField.fieldName" placeholder="请输入字段名称" />
        </el-form-item>
        <el-form-item label="字段类型" prop="fieldType">
          <el-select v-model="currentEditField.fieldType" placeholder="请选择字段类型" style="width: 100%">
            <el-option v-for="item in fieldTypeOptions" :key="item.value" :label="item.label" :value="item.value" />
          </el-select>
        </el-form-item>
        <template v-if="!isContainerType(currentEditField.fieldType)">
          <el-form-item v-if="currentEditField.fieldType" label="有效值">
            <div v-if="currentEditField.fieldType === FieldTypeEnum.Enum">
              <div class="possible-values-table">
                <el-table :data="editPossibleValuesArray" border size="small" max-height="200">
                  <el-table-column label="值" min-width="120">
                    <template #default="scope">
                      <el-input-number v-model="scope.row.value"
                                       :min="0"
                                       placeholder="值"
                                       size="small"
                                       style="width: 100%" />
                    </template>
                  </el-table-column>
                  <el-table-column label="概率(%)" min-width="120">
                    <template #default="scope">
                      <el-input-number v-model="scope.row.probability"
                                       :min="0"
                                       :max="100"
                                       placeholder="概率"
                                       size="small"
                                       style="width: 100%" />
                    </template>
                  </el-table-column>
                  <el-table-column label="操作" width="80" align="center">
                    <template #default="scope">
                      <el-button type="danger" circle size="small" @click="removeEditPossibleValue(scope.$index)">
                        <el-icon>
                          <delete />
                        </el-icon>
                      </el-button>
                    </template>
                  </el-table-column>
                </el-table>
                <div class="table-actions">
                  <el-button type="primary" size="small" @click="addEditPossibleValue">
                    添加可能值
                  </el-button>
                </div>
              </div>
            </div>
            <div v-else>
              <!-- 非枚举类型只有一个值 -->
              <el-input v-model="editSingleDefaultValue" placeholder="有效值" style="width: 100%" />
            </div>
          </el-form-item>
          <template v-if="isNumericType(currentEditField.fieldType)">
            <el-form-item label="最小值">
              <el-input-number v-model="currentEditField.minValue"
                               :precision="currentEditField.fieldType === FieldTypeEnum.Float ? 2 : 0" />
            </el-form-item>
            <el-form-item label="最大值">
              <el-input-number v-model="currentEditField.maxValue"
                               :precision="currentEditField.fieldType === FieldTypeEnum.Float ? 2 : 0" />
            </el-form-item>
          </template>
        </template>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="editFieldDialogVisible = false">取消</el-button>
          <el-button type="primary" @click="confirmEditField">确认</el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>
<script setup lang="ts" name="DataModel">
import { convertToJsonStruct, convertJsonStructToJson } from "@/hooks/processJsonStruct"
import { FieldTypeEnum, JsonStruct, PossibleValue } from "@/types/mqttConfig"
import { open } from "@tauri-apps/plugin-dialog"
import { invoke } from "@tauri-apps/api/core"
import { ElMessage, FormInstance } from "element-plus"
import { Refresh, Plus, Download, Delete } from "@element-plus/icons-vue"
import { useWindowSize } from "@vueuse/core"
const { height } = useWindowSize()

const config = inject<any>("config")
const loading = ref<boolean>(true)
const addFieldDialogVisible = ref(false)
const editFieldDialogVisible = ref(false)
const currentEditField = ref<JsonStruct>({})
const fieldFormRef = ref<FormInstance>()
const editFieldFormRef = ref<FormInstance>()
const objectFields = ref<{ id: string; label: string }[]>([])
const selectedParentFieldId = ref<string>("")

const ModelTableHeight = computed(() => {
  return Math.max(490, height.value - 490) // 调整为更小的高度，最小520px
})

const newField = ref<JsonStruct>({
  fieldName: "",
  fieldType: FieldTypeEnum.String,
  minValue: undefined,
  maxValue: undefined,
  possibleValues: [],
})

const fieldRules = {
  fieldName: [{ required: true, message: "请输入字段名称", trigger: "blur" }],
  fieldType: [{ required: true, message: "请选择字段类型", trigger: "change" }],
}

const fieldTypeOptions = [
  { value: FieldTypeEnum.String, label: "字符串" },
  { value: FieldTypeEnum.Integer, label: "整数" },
  { value: FieldTypeEnum.Float, label: "浮点数" },
  { value: FieldTypeEnum.Boolean, label: "布尔值" },
  { value: FieldTypeEnum.Enum, label: "枚举" },
  { value: FieldTypeEnum.Array, label: "数组" },
  { value: FieldTypeEnum.Object, label: "对象" },
  { value: FieldTypeEnum.DateTime, label: "日期时间" },
  { value: FieldTypeEnum.Timestamp, label: "时间戳" },
]

const possibleValuesArray = ref<PossibleValue[]>([])
const editPossibleValuesArray = ref<PossibleValue[]>([])
const singleDefaultValue = ref<any>(undefined)
const editSingleDefaultValue = ref<any>(undefined)

// 字段类型工具函数
const isNumericType = (fieldType: FieldTypeEnum): boolean => {
  return fieldType === FieldTypeEnum.Integer || fieldType === FieldTypeEnum.Float
}

const isContainerType = (fieldType: FieldTypeEnum): boolean => {
  return fieldType === FieldTypeEnum.Object || fieldType === FieldTypeEnum.Array
}

const getDefaultValueByType = (fieldType: FieldTypeEnum): any => {
  switch (fieldType) {
    case FieldTypeEnum.Integer:
    case FieldTypeEnum.Float:
    case FieldTypeEnum.Timestamp:
      return 0
    case FieldTypeEnum.Boolean:
      return false
    case FieldTypeEnum.String:
    case FieldTypeEnum.DateTime:
    default:
      return ""
  }
}

// 根据字段类型转换值
const convertValueByFieldType = (value: any, fieldType: FieldTypeEnum): any => {
  switch (fieldType) {
    case FieldTypeEnum.Integer:
      return Number.isInteger(Number(value)) ? Number(value) : Math.floor(Number(value))
    case FieldTypeEnum.Float:
    case FieldTypeEnum.Timestamp:
      return Number(value)
    case FieldTypeEnum.Boolean:
      return Boolean(value)
    case FieldTypeEnum.String:
    case FieldTypeEnum.DateTime:
      return String(value)
    case FieldTypeEnum.Enum:
    default:
      return value
  }
}

// 转换possibleValues数组中的值类型
const convertPossibleValuesType = (
  possibleValues: PossibleValue[],
  fieldType: FieldTypeEnum,
): PossibleValue[] => {
  return possibleValues.map((item) => ({
    value: convertValueByFieldType(item.value, fieldType),
    probability: item.probability,
  }))
}

// 字段操作工具函数
const resetFieldNumericProps = (field: JsonStruct): void => {
  if (!isNumericType(field.fieldType)) {
    field.minValue = undefined
    field.maxValue = undefined
  }
}

const processFieldPossibleValues = (
  field: JsonStruct,
  singleValue: any,
  enumValues: PossibleValue[],
): void => {
  if (field.fieldType === FieldTypeEnum.Object) {
    field.children = field.children || []
    field.possibleValues = []
  } else if (field.fieldType === FieldTypeEnum.Enum) {
    field.possibleValues = convertPossibleValuesType([...enumValues], field.fieldType)
  } else if (!isContainerType(field.fieldType)) {
    const defaultValue =
      singleValue !== undefined && singleValue !== ""
        ? singleValue
        : getDefaultValueByType(field.fieldType)

    field.possibleValues = [
      {
        value: convertValueByFieldType(defaultValue, field.fieldType),
        probability: 100,
      },
    ]
  }

  resetFieldNumericProps(field)
}

// 表单清理和初始化函数
const clearFormData = (isEdit: boolean = false): void => {
  if (isEdit) {
    editPossibleValuesArray.value = []
    editSingleDefaultValue.value = undefined
  } else {
    possibleValuesArray.value = []
    singleDefaultValue.value = undefined
  }
}

const initializeDefaultValueByType = (fieldType: FieldTypeEnum, isEdit: boolean = false): void => {
  const defaultValue = getDefaultValueByType(fieldType)

  if (isEdit) {
    editSingleDefaultValue.value = defaultValue
  } else {
    singleDefaultValue.value = defaultValue
  }
}

// 添加可能值（创建表单）
const addPossibleValue = () => {
  possibleValuesArray.value.push({ value: 0, probability: 0 })
}

// 移除可能值（创建表单）
const removePossibleValue = (index: number) => {
  possibleValuesArray.value.splice(index, 1)
}

// 添加可能值（编辑表单）
const addEditPossibleValue = () => {
  editPossibleValuesArray.value.push({ value: 0, probability: 0 })
}

// 移除可能值（编辑表单）
const removeEditPossibleValue = (index: number) => {
  editPossibleValuesArray.value.splice(index, 1)
}

// 获取字段类型名称
const getFieldTypeName = (type: FieldTypeEnum): string => {
  const option = fieldTypeOptions.find((opt) => opt.value === type)
  return option ? option.label : "未知类型"
}

// 获取所有Object类型字段
const getAllObjectFields = (
  fields: JsonStruct[],
  prefix: string = "",
): { id: string; label: string }[] => {
  let result: { id: string; label: string }[] = []

  fields.forEach((field) => {
    const fullPath = prefix ? `${prefix}.${field.fieldName}` : field.fieldName

    if (field.fieldType === FieldTypeEnum.Object) {
      result.push({
        id: field.fieldName,
        label: fullPath,
      })

      // 递归获取子对象字段
      if (field.children && field.children.length > 0) {
        result = result.concat(getAllObjectFields(field.children, fullPath))
      }
    }
  })

  return result
}

// 刷新数据结构
const refreshStructure = () => {
  loading.value = true
  let parsedData = {}
  try {
    parsedData = JSON.parse(config.value.sendData || "{}")
  } catch (e) {
    console.error(e)
    ElMessage.error("JSON格式错误，无法解析")
  }
  config.value.fieldStruct = convertToJsonStruct(parsedData, [])
  updateJsonFromStruct()
  loading.value = false
  ElMessage.success("数据结构已刷新")
}

// 更新JSON数据
const updateJsonFromStruct = () => {
  config.value.sendData = JSON.stringify(convertJsonStructToJson(config.value.fieldStruct), null, 2)
}

// 显示添加字段对话框
const showAddFieldDialog = () => {
  newField.value = {
    fieldName: "",
    fieldType: FieldTypeEnum.String,
    minValue: undefined,
    maxValue: undefined,
    possibleValues: [],
    children: [],
  }

  possibleValuesArray.value = []
  singleDefaultValue.value = undefined

  // 获取所有Object类型字段
  objectFields.value = getAllObjectFields(config.value.fieldStruct || [])
  selectedParentFieldId.value = ""

  addFieldDialogVisible.value = true
}

// 控制是否是编辑对话框初始化阶段
const isEditDialogInitializing = ref(false)

// 显示编辑字段对话框
const showEditFieldDialog = (row: JsonStruct) => {
  isEditDialogInitializing.value = true

  currentEditField.value = JSON.parse(JSON.stringify(row))

  // 使用 nextTick 确保响应式更新完成后再设置值
  nextTick(() => {
    // 初始化编辑表单中的可能值数组
    if (currentEditField.value.fieldType === FieldTypeEnum.Enum) {
      editPossibleValuesArray.value = [...(currentEditField.value.possibleValues || [])]
      editSingleDefaultValue.value = undefined
    } else if (!isContainerType(currentEditField.value.fieldType)) {
      // 对于非容器类型，回显第一个有效值或设置默认值
      if (currentEditField.value.possibleValues?.length) {
        editSingleDefaultValue.value = currentEditField.value.possibleValues[0]?.value
      } else {
        editSingleDefaultValue.value = getDefaultValueByType(currentEditField.value.fieldType)
      }
      editPossibleValuesArray.value = []
    } else {
      // 容器类型 (Object 或 Array)
      clearFormData(true)
    }

    isEditDialogInitializing.value = false
  })

  editFieldDialogVisible.value = true
}

// 确认添加字段前处理possibleValues
const confirmAddField = () => {
  fieldFormRef.value?.validate((valid) => {
    if (valid) {
      const fieldToAdd = { ...newField.value }

      // 使用工具函数处理字段的可能值和属性
      processFieldPossibleValues(fieldToAdd, singleDefaultValue.value, possibleValuesArray.value)

      // 根据选择的父字段确定添加位置
      if (selectedParentFieldId.value) {
        // 添加到指定的Object字段的children中
        const findAndAddToParent = (arr: JsonStruct[], parentId: string): boolean => {
          for (let i = 0; i < arr.length; i++) {
            if (arr[i].fieldName === parentId && arr[i].fieldType === FieldTypeEnum.Object) {
              if (!arr[i].children) {
                arr[i].children = []
              }
              arr[i].children!.push(fieldToAdd)
              return true
            }

            // 递归查找子对象
            if (arr[i].children && findAndAddToParent(arr[i].children, parentId)) {
              return true
            }
          }
          return false
        }

        if (!findAndAddToParent(config.value.fieldStruct, selectedParentFieldId.value)) {
          ElMessage.error("未找到指定的父字段")
          return
        }
      } else {
        // 添加为根级字段
        config.value.fieldStruct.push(fieldToAdd)
      }

      addFieldDialogVisible.value = false
      updateJsonFromStruct()
      ElMessage.success("字段添加成功")
    }
  })
}

// 确认编辑字段前处理possibleValues
const confirmEditField = () => {
  editFieldFormRef.value?.validate((valid) => {
    if (valid) {
      // 使用工具函数处理字段的可能值和属性
      processFieldPossibleValues(
        currentEditField.value,
        editSingleDefaultValue.value,
        editPossibleValuesArray.value,
      )

      // 找到字段并更新（支持嵌套查找）
      const updateField = (arr: JsonStruct[], fieldId: string): boolean => {
        for (let i = 0; i < arr.length; i++) {
          if (arr[i].fieldName === fieldId) {
            // 更新字段属性，保留minValue和maxValue
            arr[i] = { ...currentEditField.value }
            return true
          }

          // 递归查找子字段
          if (arr[i].children && updateField(arr[i].children, fieldId)) {
            return true
          }
        }
        return false
      }

      updateField(config.value.fieldStruct, currentEditField.value.fieldName)
      editFieldDialogVisible.value = false
      updateJsonFromStruct()
      ElMessage.success("字段更新成功")
    }
  })
}

// 导入模板
const importTemplate = async () => {
  try {
    const filePath = (await open({
      multiple: false,
      filters: [{ name: "JSON", extensions: ["json"] }],
    })) as string

    if (filePath) {
      const templateContent = await invoke("read_file", {
        path: filePath,
      })
      try {
        const parsedTemplate = JSON.parse(templateContent as string)
        config.value.fieldStruct = convertToJsonStruct(parsedTemplate, [])
        config.value.sendData = JSON.stringify(parsedTemplate, null, 2)
        ElMessage.success("模板导入成功")
      } catch (e) {
        console.error(e)
        ElMessage.error("模板文件格式错误")
      }
    }
  } catch (error) {
    ElMessage.error(`导入失败: ${error}`)
  }
}

watch(
  () => config.value.fieldStruct,
  (newVal) => {
    if (newVal === undefined) return
    config.value.fieldStruct = newVal
    updateJsonFromStruct()
  },
  { deep: true },
)

onMounted(() => {
  loading.value = true

  // 如果已经有fieldStruct配置，则不需要重新从sendData解析
  if (config.value.fieldStruct && config.value.fieldStruct.length > 0) {
    loading.value = false
    return
  }

  // 只有当没有fieldStruct时，才从sendData解析生成初始结构
  let parsedData = {}
  try {
    parsedData = JSON.parse(config.value.sendData || "{}")
  } catch (e) {
    console.error(e)
    ElMessage.error("JSON格式错误，无法解析")
  }
  config.value.fieldStruct = convertToJsonStruct(parsedData, [])
  loading.value = false
})

// 在处理编辑对话框时，字段类型切换时的处理
watch(
  () => currentEditField.value.fieldType,
  (newType) => {
    // 如果是对话框初始化阶段，跳过watcher处理
    if (isEditDialogInitializing.value) {
      return
    }

    // 清空表单数据
    clearFormData(true)
    resetFieldNumericProps(currentEditField.value)
    currentEditField.value.possibleValues = []

    // 根据新类型设置合适的默认值
    if (newType === FieldTypeEnum.Enum) {
      // 枚举类型不需要单个默认值
    } else if (!isContainerType(newType)) {
      // 为非容器类型设置默认值
      initializeDefaultValueByType(newType, true)
    }

    // 如果是Object类型，初始化children
    if (newType === FieldTypeEnum.Object) {
      currentEditField.value.children = []
    } else {
      currentEditField.value.children = undefined
    }
  },
)

// 在处理新建字段时，字段类型切换时清空默认值表单
watch(
  () => newField.value.fieldType,
  (newType) => {
    // 清空表单数据
    clearFormData(false)
    resetFieldNumericProps(newField.value)
    newField.value.possibleValues = []

    // 如果是Object类型，初始化children
    if (newType === FieldTypeEnum.Object) {
      newField.value.children = []
    } else {
      newField.value.children = undefined
    }
  },
)
</script>
<style lang="scss" scoped>
// 特定于DataModel的样式
.data-model {
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;

    .title {
      font-size: 14px;
      font-weight: 500;
      color: var(--el-text-color-primary);
    }

    .actions {
      display: flex;
      gap: 6px;

      .el-button {
        width: 28px;
        height: 28px;

        .el-icon {
          font-size: 12px;
        }
      }
    }
  }

  .table-container {
    margin-bottom: 16px;
  }
}
</style>
