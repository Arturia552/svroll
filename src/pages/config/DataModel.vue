<template>
  <div v-loading="loading" class="data-model">
    <div class="header">
      <div class="title">
        数据模型配置
      </div>
      <div class="actions">
        <el-tooltip content="刷新数据结构" placement="top">
          <el-button type="primary" circle @click="refreshStructure">
            <el-icon>
              <refresh />
            </el-icon>
          </el-button>
        </el-tooltip>
        <el-tooltip content="添加字段" placement="top">
          <el-button type="success" circle @click="showAddFieldDialog">
            <el-icon>
              <plus />
            </el-icon>
          </el-button>
        </el-tooltip>
        <el-tooltip content="导入模板" placement="top">
          <el-button type="info" circle @click="importTemplate">
            <el-icon>
              <download />
            </el-icon>
          </el-button>
        </el-tooltip>
      </div>
    </div>

    <el-divider />

    <div class="table-container">
      <el-table :data="config.fieldStruct" style="width: 100%" size="small" row-key="fieldName"
                :tree-props="{ children: 'children' }" border stripe highlight-current-row
      >
        <el-table-column prop="fieldName" label="键值" min-width="20%" />

        <el-table-column prop="fieldType" label="数据类型" min-width="20%">
          <template #default="scope">
            <el-select v-model="scope.row.fieldType" v-focus
                       class="edit-select" size="small"
                       @change="handleTypeChange"
            >
              <el-option v-for="item in fieldTypeOptions" 
                         :key="item.value" 
                         :label="item.label" 
                         :value="item.value"
              />
            </el-select>
          </template>
        </el-table-column>

        <el-table-column prop="minValue" label="最小值" min-width="15%">
          <template #default="scope">
            <on-click-outside v-if="scope.row.editing1 && scope.row.fieldType !== FieldTypeEnum.Object"
                              @trigger="scope.row.editing1 = false"
            >
              <el-input v-if="scope.row.editing1" ref="minValueInput" v-model="scope.row.minValue" v-focus
                        class="edit-input" @blur="
                          blurInput(scope.row, scope.row.minValue, 'minValue', 'editing1')
                        "
              />
            </on-click-outside>
            <div v-else class="editable-cell" @dblclick="activateEdit(scope.row, 'editing1')">
              {{ scope.row.minValue ?? "--" }}
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="maxValue" label="最大值" min-width="15%">
          <template #default="scope">
            <on-click-outside v-if="scope.row.editing2 && scope.row.fieldType !== FieldTypeEnum.Object"
                              @trigger="scope.row.editing2 = false"
            >
              <el-input v-if="scope.row.editing2" v-model="scope.row.maxValue" v-focus class="edit-input" @blur="
                blurInput(scope.row, scope.row.maxValue, 'maxValue', 'editing2')
              "
              />
            </on-click-outside>
            <div v-else class="editable-cell" @dblclick="activateEdit(scope.row, 'editing2')">
              {{ scope.row.maxValue ?? "--" }}
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="possibleValues" label="有效值" min-width="20%">
          <template #default="scope">
            <on-click-outside v-if="
              scope.row.editing3 && scope.row.fieldType !== FieldTypeEnum.Object
            " @trigger="scope.row.editing3 = false"
            >
              <el-input v-if="scope.row.editing3" v-model="scope.row.possibleValues" v-focus class="edit-input" @blur="
                blurInput(
                  scope.row,
                  scope.row.possibleValues,
                  'possibleValues',
                  'editing3'
                )
              "
              />
            </on-click-outside>
            <div v-else class="editable-cell" @dblclick="activateEdit(scope.row, 'editing3')">
              {{ scope.row.possibleValues ?? "--" }}
            </div>
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

    <div class="tips">
      <el-alert type="info" :closable="false">
        <div class="tips-content">
          <p><strong>提示：</strong> 双击单元格进行编辑。</p>
        </div>
      </el-alert>
    </div>

    <!-- 添加字段对话框 -->
    <el-dialog v-model="addFieldDialogVisible" title="添加字段" width="500px" :close-on-click-modal="false"
               :append-to-body="true"
    >
      <el-form ref="fieldFormRef" :model="newField" label-width="100px" :rules="fieldRules">
        <el-form-item label="添加到" prop="parentField">
          <el-select v-model="selectedParentFieldId" placeholder="请选择添加位置" style="width: 100%">
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
        <template v-if="newField.fieldType !== FieldTypeEnum.Object && newField.fieldType !== FieldTypeEnum.Array">
          <el-form-item v-if="newField.fieldType" label="默认值">
            <el-input v-model="newField.possibleValues" placeholder="请输入默认值" />
          </el-form-item>
          <template v-if="newField.fieldType === FieldTypeEnum.Integer || newField.fieldType === FieldTypeEnum.Float">
            <el-form-item label="最小值">
              <el-input-number v-model="newField.minValue"
                               :precision="newField.fieldType === FieldTypeEnum.Float ? 2 : 0"
              />
            </el-form-item>
            <el-form-item label="最大值">
              <el-input-number v-model="newField.maxValue"
                               :precision="newField.fieldType === FieldTypeEnum.Float ? 2 : 0"
              />
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
    <el-dialog v-model="editFieldDialogVisible" title="编辑字段" width="500px" :close-on-click-modal="false"
               :append-to-body="true"
    >
      <el-form ref="editFieldFormRef" :model="currentEditField" label-width="80px" :rules="fieldRules">
        <el-form-item label="字段名称" prop="fieldName">
          <el-input v-model="currentEditField.fieldName" placeholder="请输入字段名称" />
        </el-form-item>
        <el-form-item label="字段类型" prop="fieldType">
          <el-select v-model="currentEditField.fieldType" placeholder="请选择字段类型" style="width: 100%">
            <el-option v-for="item in fieldTypeOptions" :key="item.value" :label="item.label" :value="item.value" />
          </el-select>
        </el-form-item>
        <template
          v-if="currentEditField.fieldType !== FieldTypeEnum.Object && currentEditField.fieldType !== FieldTypeEnum.Array"
        >
          <el-form-item v-if="currentEditField.fieldType" label="默认值">
            <el-input v-model="currentEditField.possibleValues" placeholder="请输入默认值" />
          </el-form-item>
          <template
            v-if="currentEditField.fieldType === FieldTypeEnum.Integer || currentEditField.fieldType === FieldTypeEnum.Float"
          >
            <el-form-item label="最小值">
              <el-input-number v-model="currentEditField.minValue"
                               :precision="currentEditField.fieldType === FieldTypeEnum.Float ? 2 : 0"
              />
            </el-form-item>
            <el-form-item label="最大值">
              <el-input-number v-model="currentEditField.maxValue"
                               :precision="currentEditField.fieldType === FieldTypeEnum.Float ? 2 : 0"
              />
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
import { convertToJsonStruct } from "@/hooks/processJsonStruct";
import { FieldTypeEnum, JsonStruct } from "@/types/mqttConfig";
import { OnClickOutside } from "@vueuse/components";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { ElMessage, FormInstance } from "element-plus";
import { Refresh, Plus, Download } from '@element-plus/icons-vue';

const config = inject<any>("config");
const loading = ref<boolean>(true);
const addFieldDialogVisible = ref(false);
const editFieldDialogVisible = ref(false);
const currentParent = ref<JsonStruct | null>(null);
const currentEditField = ref<JsonStruct>({});
const fieldFormRef = ref<FormInstance>();
const editFieldFormRef = ref<FormInstance>();
const objectFields = ref<{ id: string, label: string }[]>([]);
const selectedParentFieldId = ref<string>('');

// 自定义指令：自动聚焦
const vFocus = {
  mounted: (el) => {
    el.querySelector('input')?.focus();
  }
};

const newField = ref<JsonStruct>({
  fieldName: '',
  fieldType: FieldTypeEnum.String,
  minValue: undefined,
  maxValue: undefined,
  possibleValues: '',
  children: []
});

const fieldRules = {
  fieldName: [
    { required: true, message: '请输入字段名称', trigger: 'blur' },
    { pattern: /^[a-zA-Z][a-zA-Z0-9_]*$/, message: '字段名称只能包含字母、数字和下划线，且必须以字母开头', trigger: 'blur' }
  ],
  fieldType: [
    { required: true, message: '请选择字段类型', trigger: 'change' }
  ]
};

const fieldTypeOptions = [
  { value: FieldTypeEnum.String, label: '字符串(String)' },
  { value: FieldTypeEnum.Integer, label: '整数(Integer)' },
  { value: FieldTypeEnum.Float, label: '浮点数(Float)' },
  { value: FieldTypeEnum.Boolean, label: '布尔值(Boolean)' },
  { value: FieldTypeEnum.Object, label: '对象(Object)' },
  { value: FieldTypeEnum.Array, label: '数组(Array)' },
  { value: FieldTypeEnum.DateTime, label: '日期时间(DateTime)' },
  { value: FieldTypeEnum.Timestamp, label: '时间戳(Timestamp)' }
];

const convertJsonStructToJson = (jsonStructArray: JsonStruct[]): object => {
  const result: any = {};
  jsonStructArray.forEach((item) => {
    if (item.fieldType === FieldTypeEnum.Object && item.children?.length) {
      result[item.fieldName] = convertJsonStructToJson(item.children);
    } else if (item.fieldType === FieldTypeEnum.Array) {
      result[item.fieldName] = [];
    } else {
      result[item.fieldName] = item.possibleValues;
    }
  });
  return result;
};

const blurInput = (
  row: JsonStruct,
  val: any,
  valueKey: string,
  editKey: string
) => {
  row[editKey] = false;
  if (row.fieldType === FieldTypeEnum.Object) return;
  else if (row.fieldType === FieldTypeEnum.String) {
    row[valueKey] = val;
  } else if (row.fieldType === FieldTypeEnum.Integer) {
    const parsed = parseInt(val);
    row[valueKey] = isNaN(parsed) ? 0 : parsed;
  } else if (row.fieldType === FieldTypeEnum.Float) {
    const parsed = parseFloat(val);
    row[valueKey] = isNaN(parsed) ? 0 : parsed;
  } else if (row.fieldType === FieldTypeEnum.Boolean) {
    row[valueKey] = val === "true";
  }
};

const activateEdit = (row: JsonStruct, editKey: string) => {
  if (row.fieldType !== FieldTypeEnum.Object) {
    row[editKey] = true;
  }
};

const handleTypeChange = (row: JsonStruct) => {
  const newFieldType = row.fieldType;
  if (newFieldType === FieldTypeEnum.Object && !row.children) {
    row.children = [];
  }
  if (newFieldType === FieldTypeEnum.Object || newFieldType === FieldTypeEnum.Array) {
    row.minValue = undefined;
    row.maxValue = undefined;
    row.possibleValues = '';
  }
  
  // 更新JSON结构
  updateJsonFromStruct();
  
};

// 刷新数据结构
const refreshStructure = () => {
  loading.value = true;
  let parsedData = {};
  try {
    parsedData = JSON.parse(config.value.sendData || '{}');
  } catch (e) {
    console.error(e);
    ElMessage.error('JSON格式错误，无法解析');
  }
  config.value.fieldStruct = convertToJsonStruct(parsedData, []);
  updateJsonFromStruct();
  loading.value = false;
  ElMessage.success('数据结构已刷新');
};

// 更新JSON数据
const updateJsonFromStruct = () => {
  config.value.sendData = JSON.stringify(convertJsonStructToJson(config.value.fieldStruct), null, 2);
};

// 获取所有Object类型字段
const getObjectFields = (fields: JsonStruct[], prefix = '', path = ''): void => {
  fields.forEach((field, index) => {
    if (field.fieldType === FieldTypeEnum.Object) {
      const id = path ? `${path}.${index}` : `${index}`;
      const label = prefix ? `${prefix}.${field.fieldName}` : field.fieldName;
      objectFields.value.push({ id, label });

      if (field.children && field.children.length > 0) {
        getObjectFields(field.children, label, id);
      }
    }
  });
};

// 根据path获取字段
const getFieldByPath = (path: string): JsonStruct | null => {
  if (!path) return null;

  const indices = path.split('.').map(Number);
  let current = config.value.fieldStruct;
  let field = null;

  for (const index of indices) {
    field = current[index];
    current = field.children || [];
  }

  return field;
};

// 显示添加字段对话框
const showAddFieldDialog = () => {
  newField.value = {
    fieldName: '',
    fieldType: FieldTypeEnum.String,
    minValue: undefined,
    maxValue: undefined,
    possibleValues: '',
    children: []
  };

  // 获取所有Object类型字段
  objectFields.value = [];
  getObjectFields(config.value.fieldStruct);

  selectedParentFieldId.value = '';
  currentParent.value = null;
  addFieldDialogVisible.value = true;
};

// 确认添加字段
const confirmAddField = () => {
  fieldFormRef.value?.validate((valid) => {
    if (valid) {
      const fieldToAdd = { ...newField.value };

      // 如果是对象类型，确保有children数组
      if (fieldToAdd.fieldType === FieldTypeEnum.Object && !fieldToAdd.children) {
        fieldToAdd.children = [];
      }

      // 根据选择的父字段确定添加位置
      if (selectedParentFieldId.value) {
        const parentField = getFieldByPath(selectedParentFieldId.value);
        if (parentField) {
          if (!parentField.children) {
            parentField.children = [];
          }
          parentField.children.push(fieldToAdd);
        }
      } else {
        // 添加为根级字段
        config.value.fieldStruct.push(fieldToAdd);
      }

      addFieldDialogVisible.value = false;
      updateJsonFromStruct();
      ElMessage.success('字段添加成功');
    }
  });
};

// 确认编辑字段
const confirmEditField = () => {
  editFieldFormRef.value?.validate((valid) => {
    if (valid) {
      // 找到字段并更新
      const updateField = (arr: JsonStruct[], fieldId: string) => {
        for (let i = 0; i < arr.length; i++) {
          if (arr[i].fieldName === fieldId) {
            // 更新字段属性，保留children
            const children = arr[i].children;
            arr[i] = { ...currentEditField.value };
            if (children && arr[i].fieldType === FieldTypeEnum.Object) {
              arr[i].children = children;
            }
            return true;
          }

          if (arr[i].children && arr[i].children.length) {
            if (updateField(arr[i].children, fieldId)) {
              return true;
            }
          }
        }
        return false;
      };

      updateField(config.value.fieldStruct, currentEditField.value.fieldName);
      editFieldDialogVisible.value = false;
      updateJsonFromStruct();
      ElMessage.success('字段更新成功');
    }
  });
};

// 导入模板
const importTemplate = async () => {
  try {
    const filePath = await open({
      multiple: false,
      filters: [{ name: 'JSON', extensions: ['json'] }]
    }) as string;

    if (filePath) {
      const templateContent = await invoke('read_file', { path: filePath });
      try {
        const parsedTemplate = JSON.parse(templateContent as string);
        config.value.fieldStruct = convertToJsonStruct(parsedTemplate, []);
        config.value.sendData = JSON.stringify(parsedTemplate, null, 2);
        ElMessage.success('模板导入成功');
      } catch (e) {
        console.error(e);
        ElMessage.error('模板文件格式错误');
      }
    }
  } catch (error) {
    ElMessage.error(`导入失败: ${error}`);
  }
};

watch(() => config.value.fieldStruct, (newVal) => {
  if (newVal === undefined) return;
  config.value.fieldStruct = newVal;
  updateJsonFromStruct();
}, { deep: true });

onMounted(() => {
  loading.value = true;
  let parsedData = {};
  try {
    parsedData = JSON.parse(config.value.sendData || '{}');
  } catch (e) {
    console.error(e);
    ElMessage.error('JSON格式错误，无法解析');
  }
  config.value.fieldStruct = convertToJsonStruct(parsedData, config.value.fieldStruct || []);
  loading.value = false;
});
</script>
<style lang="scss" scoped>
.data-model {
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;

    .title {
      font-size: 18px;
      font-weight: bold;
      color: var(--el-color-primary);
    }

    .actions {
      display: flex;
      gap: 10px;
    }
  }

  .table-container {
    margin-bottom: 20px;
    border-radius: var(--el-border-radius-base);
    overflow: hidden;
    box-shadow: var(--el-box-shadow-light);

    :deep(.el-table__row) {
      transition: all 0.2s;

      &:hover {
        background-color: var(--el-fill-color-light) !important;
      }
    }
  }

  .field-name-cell {
    display: flex;
    align-items: center;
    gap: 8px;

    .el-icon {
      font-size: 16px;
      color: var(--el-text-color-regular);
    }

    span {
      font-weight: 500;
    }
  }

  .editable-cell {
    display: block;
    width: 100%;
    padding: 6px 8px;
    border-radius: var(--el-border-radius-small);
    cursor: pointer;
    transition: all 0.2s;

    &:hover {
      background-color: var(--el-color-primary-light-9);
    }
  }

  .edit-input {
    width: 100%;

    :deep(.el-input__inner) {
      padding: 6px 8px;
    }
  }

  .actions-cell {
    display: flex;
    justify-content: center;

    .el-button {
      padding: 6px;
      margin: 0 2px;
    }
  }

  .tips {
    margin-top: 20px;

    .tips-content {
      p {
        margin: 8px 0;
        line-height: 1.5;
      }

      .el-tag {
        margin-right: 8px;
      }
    }
  }
}
</style>
