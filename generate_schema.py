import json
import re
import typing
from pathlib import Path
from typing import Dict, Any, Tuple

# language=rust
HEAD = """
use crate::reporter::Reporter;
use crate::visit::OparlObject;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Deref;

/// Url linking to another oparl object
#[derive(Debug, Serialize, Deserialize)]
pub struct OparlUrl<T>(String, PhantomData<T>);

impl<T> Deref for OparlUrl<T> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Url linking to a list or an external resource
#[derive(Debug, Serialize, Deserialize)]
pub struct OtherUrl(String);

impl Deref for OtherUrl {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

"""


def add_field(fp: typing.TextIO, description: Dict[str, Any]) -> str:
    match description["type"]:
        case "string":
            if description.get("format") == "url":
                if ref := description.get("oparl:ref"):
                    return f"OparlUrl<{ref}>"
                else:
                    return f"OtherUrl"
            else:
                return f"String"
        case "array":
            # Hack to not loose ref
            inner_description = {**description, **description["items"]}
            return f"Vec<{add_field(fp, inner_description)}>"
        case "boolean":
            return "bool"
        case "integer":
            return "usize"
        case "object":
            if title := description.get("title"):
                add_struct(fp, description)
                return title
            elif ref := description.get("$ref"):
                return ref.split(".")[0]
            else:
                return "HashMap<String, Value>"
        case _:
            print(f"Warning: Unknown type: {description['type']}")
            return "Value"


def write_struct(
    fp: typing.TextIO,
    rust_fields: typing.List[Tuple[str, str, str]],
    schema,
):
    fp.write("#[derive(Debug, Serialize, Deserialize)]\n")
    fp.write('#[serde(rename_all = "camelCase")]\n')
    fp.write(f"pub struct {schema['title']} {{\n")
    for _, snake_case_key, rust_type in rust_fields:
        fp.write(f"    pub {snake_case_key}: {rust_type},\n")
    fp.write("    #[serde(flatten)]\n")
    fp.write("    pub other: HashMap<String, Value>,\n")
    fp.write("}\n\n")
    fp.write(f"impl OparlObject for {schema['title']} {{\n")
    fp.write("    fn type_name() -> &'static str {\n")
    fp.write(f'        "{schema["title"]}"\n')
    fp.write("    }\n")
    fp.write("\n")
    fp.write("    fn visit_fields(&self, reporter: &impl Reporter, url: &str) {\n")
    for key, snake_case_key, _ in rust_fields:
        fp.write(
            f'        self.visit_field(reporter, "{key}", &self.{snake_case_key}, &url);\n'
        )
    fp.write("    }\n")
    fp.write("\n")
    fp.write("    fn get_required(&self) -> Vec<&str> {\n")
    fp.write('        vec!["' + '", "'.join(schema["required"]) + '"]\n')
    fp.write("    }\n")
    fp.write("\n")
    fp.write("    fn get_id(&self) -> Option<&str> {\n")
    fp.write("        self.id.as_ref().map(|x| x.as_str())\n")
    fp.write("    }\n")
    fp.write("\n")
    fp.write("}\n\n")


def add_struct(fp: typing.TextIO, schema: Dict[str, Any]):
    # https://stackoverflow.com/a/1176023/3549270
    camel_case_to_snake_case = re.compile(r"(?<!^)(?=[A-Z])")
    rust_fields = []
    for key, json_property in schema["properties"].items():
        snake_case_key = camel_case_to_snake_case.sub("_", key).lower()
        if snake_case_key == "type":
            snake_case_key = "r#type"
        rust_type = add_field(fp, json_property)
        # if key not in schema["required"]:
        # Most haven't even gotten the mandatory legislativeTerm
        rust_type = f"Option<{rust_type}>"
        rust_fields.append((key, snake_case_key, rust_type))
    write_struct(fp, rust_fields, schema)


def write_schema(fp: typing.TextIO):
    fp.write(HEAD.lstrip())

    for file in Path("oparl/schema").glob("*.json"):
        schema = json.loads(file.read_text())
        add_struct(fp, schema)


def main():
    with Path("src/schema.rs").open("w") as fp:
        fp.write(HEAD.lstrip())
        for file in Path("oparl/schema").glob("*.json"):
            schema = json.loads(file.read_text())
            add_struct(fp, schema)


if __name__ == "__main__":
    main()
