// src/components/detail/FieldDetail.tsx
import type { FieldRow } from "../../types/ddr";
import { FieldBasicProperties } from "./field/FieldBasicProperties";
import { FieldAutoEnter } from "./field/FieldAutoEnter";
import { FieldValidationRules } from "./field/FieldValidationRules";
import { FieldStorage } from "./field/FieldStorage";
import { FieldCalcReferences } from "./field/FieldCalcReferences";
import { FieldScriptReferences } from "./field/FieldScriptReferences";
import { FieldLayoutReferences } from "./field/FieldLayoutReferences";
import { FieldRelationshipReferences } from "./field/FieldRelationshipReferences";

interface Props {
  field: FieldRow;
  tableName: string;
  projectId: number;
}

export function FieldDetail({ field, tableName, projectId }: Props) {
  return (
    <div className="p-4 space-y-4 text-sm">
      <FieldBasicProperties field={field} />
      <FieldAutoEnter field={field} />
      <FieldValidationRules field={field} />
      <FieldStorage field={field} />
      <FieldCalcReferences
        projectId={projectId}
        tableName={tableName}
        fieldName={field.name}
      />
      <FieldScriptReferences
        projectId={projectId}
        tableName={tableName}
        fieldName={field.name}
      />
      <FieldLayoutReferences
        projectId={projectId}
        tableName={tableName}
        fieldName={field.name}
      />
      <FieldRelationshipReferences
        projectId={projectId}
        tableName={tableName}
        fieldName={field.name}
      />
    </div>
  );
}
