import re

with open('commands.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace('let planned = document_from_generation_plan_text(&source_text)?;\n', 'let document = document_from_generation_plan_text(&source_text)?;\n')
text = text.replace('let revision = document_revision_id(&planned.document);', 'let revision = document_revision_id(&document);')
text = text.replace('planned.operation_count', '0')
text = text.replace('planned.summary.subject_classification', '\"object\".to_string()')
text = text.replace('planned.summary.subject_label', '\"generated\".to_string()')
text = text.replace('document: planned.document,', 'document: document.clone(),')
text = re.sub(r'plan_summary:\s*planned\.summary,\s*', '', text)
text = re.sub(r'operation_count:\s*planned\.operation_count,\s*', '', text)
text = text.replace('planned_doc.document', 'planned_doc')

with open('commands.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print('Patched commands.rs again')
