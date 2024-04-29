mod cache;
mod config;
mod utils;

use std::collections::HashMap;

use cache::Cache;
use config::Config;
use swc_core::{
    common::{util::take::Take, DUMMY_SP},
    ecma::{
        ast::{
            ArrayLit, ArrowExpr, BindingIdent, BlockStmtOrExpr, CallExpr, Callee, Decl, Expr,
            ExprOrSpread, Ident, JSXAttrName, JSXAttrOrSpread, JSXAttrValue, JSXClosingElement,
            JSXClosingFragment, JSXElement, JSXElementChild, JSXElementName, JSXExpr, JSXFragment,
            JSXOpeningElement, JSXOpeningFragment, KeyValueProp, Lit, ModuleDecl, ModuleItem,
            ObjectLit, Pat, Program, Prop, PropName, PropOrSpread, Stmt, Str, VarDecl,
        },
        atoms::JsWord,
        visit::{as_folder, noop_visit_mut_type, FoldWith, VisitMut, VisitMutWith},
    },
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};
use utils::{emit_error, json_path_from_key, parse_code};

const TRANSLATION_METHOD_NAME: &str = "t";
const TRANSLATION_HOOKS_NAME: &str = "useTranslation";
const TRANSLATION_COMPONENT_NAME: &str = "Trans";
const TRANSLATION_COMPONENT_ATTR_NAME: &str = "components";
const TRANSLATION_PACKAGE_NAME: &str = "react-i18next";

pub struct TransformVisitor {
    scopes: Vec<String>,
    cache: Cache,
    component_map: HashMap<String, JSXElement>,
}

impl TransformVisitor {
    pub fn new(base_dir: String) -> Self {
        Self {
            scopes: vec![],
            component_map: HashMap::new(),
            cache: Cache::new(base_dir),
        }
    }
}

impl VisitMut for TransformVisitor {
    noop_visit_mut_type!();

    fn visit_mut_module_items(&mut self, n: &mut Vec<ModuleItem>) {
        n.visit_mut_children_with(self);
        // remove invalid import statement
        n.retain(|s| !matches!(s, ModuleItem::Stmt(Stmt::Empty(..))));
    }

    fn visit_mut_module_item(&mut self, n: &mut ModuleItem) {
        n.visit_mut_children_with(self);
        // mark import statement for react-i18next as an invalid node
        // e.g) import { useTranslation } from "react-i18next";
        if let ModuleItem::ModuleDecl(ModuleDecl::Import(decl)) = &*n {
            if &*decl.src.value == TRANSLATION_PACKAGE_NAME {
                n.take();
            }
        }
    }

    fn visit_mut_stmts(&mut self, stmts: &mut Vec<Stmt>) {
        stmts.visit_mut_children_with(self);
        // remove invalid import statement
        stmts.retain(|s| !matches!(s, Stmt::Empty(..)));
    }

    fn visit_mut_stmt(&mut self, s: &mut Stmt) {
        s.visit_mut_children_with(self);

        if let Stmt::Decl(Decl::Var(var)) = &*s {
            if let Some(decl) = var.decls.get(0) {
                if let Some(ref init) = decl.init {
                    if let Expr::Call(call_expr) = &**init {
                        if let Callee::Expr(callee_expr) = &call_expr.callee {
                            if let Expr::Ident(ident) = &**callee_expr {
                                if ident.sym.to_string() == TRANSLATION_HOOKS_NAME {
                                    // mark declarations for react-i18next as an invalid node
                                    // e.g) const { t } = useTranslation("xxx");
                                    s.take();
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn visit_mut_var_decl(&mut self, n: &mut VarDecl) {
        if let Some(decl) = n.decls.get(0) {
            if let Some(ref init) = decl.init {
                if let Expr::Call(call_expr) = &**init {
                    if let Callee::Expr(callee_expr) = &call_expr.callee {
                        if let Expr::Ident(ident) = &**callee_expr {
                            if ident.sym.to_string() == TRANSLATION_HOOKS_NAME {
                                // store args of useTranslation("foo") as scopes and load json files for i18n
                                self.scopes = vec![];
                                if let Some(arg) = call_expr.args.get(0) {
                                    if let Expr::Array(lit) = &*arg.expr {
                                        // e.g) useTranslation(["foo", "bar"])
                                        for el in &lit.elems {
                                            let Some(el) = el else { return; };
                                            if let Expr::Lit(Lit::Str(Str {
                                                span,
                                                raw: _,
                                                value,
                                            })) = &*el.expr
                                            {
                                                match self.cache.add_file(value.to_string()) {
                                                    Ok(_) => self.scopes.push(value.to_string()),
                                                    Err(error) => {
                                                        emit_error(&error.to_string(), *span);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    if let Expr::Lit(Lit::Str(Str {
                                        value,
                                        span,
                                        raw: _,
                                    })) = &*arg.expr
                                    {
                                        // e.g) useTranslation("foo")
                                        match self.cache.add_file(value.to_string()) {
                                            Ok(_) => self.scopes.push(value.to_string()),
                                            Err(error) => {
                                                emit_error(&error.to_string(), *span);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        n.visit_mut_children_with(self);
    }

    fn visit_mut_jsx_element(&mut self, n: &mut JSXElement) {
        if let JSXElementName::Ident(ident) = &n.opening.name {
            if ident.sym.to_string() == TRANSLATION_COMPONENT_NAME {
                for attr in &mut n.opening.attrs.to_vec() {
                    if let JSXAttrOrSpread::JSXAttr(attr) = attr {
                        if let JSXAttrName::Ident(ident) = &mut attr.name {
                            if ident.sym.to_string() == "i18nKey" {
                                // do not support i18nKey so far
                                if let Some(JSXAttrValue::Lit(Lit::Str(_))) = &attr.value {
                                    unimplemented!("i18nKey has not been supported");
                                }
                            } else if ident.sym.to_string() == TRANSLATION_COMPONENT_ATTR_NAME {
                                if let Some(JSXAttrValue::JSXExprContainer(c)) = &mut attr.value {
                                    if let JSXExpr::Expr(expr) = &mut c.expr {
                                        if let Expr::Object(ObjectLit { span: _, props }) =
                                            &mut **expr
                                        {
                                            // store jsx attributes for mapping jsx elements
                                            // e.g) components={{ link: <Link />, button: <Button /> }}
                                            for prop in props.iter() {
                                                if let PropOrSpread::Prop(prop) = prop {
                                                    if let Prop::KeyValue(KeyValueProp {
                                                        key,
                                                        value,
                                                    }) = &**prop
                                                    {
                                                        if let Expr::JSXElement(element) = &**value
                                                        {
                                                            if let PropName::Ident(ident) = &key {
                                                                self.component_map.insert(
                                                                    ident.sym.to_string(),
                                                                    *element.clone(),
                                                                );
                                                            }
                                                        } else if let Expr::Paren(paren) = &**value
                                                        {
                                                            if let Expr::JSXElement(element) =
                                                                &*paren.expr
                                                            {
                                                                if let PropName::Ident(ident) = &key
                                                                {
                                                                    self.component_map.insert(
                                                                        ident.sym.to_string(),
                                                                        *element.clone(),
                                                                    );
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        n.visit_mut_children_with(self);
    }

    fn visit_mut_expr(&mut self, n: &mut Expr) {
        if let Expr::Call(call_expr) = &mut *n {
            // NOTE: using visit_mut_expr for replacing call_expr to literal
            if let Callee::Expr(expr) = &mut call_expr.callee {
                if let Expr::Ident(id) = &mut **expr {
                    if (&id.sym == "useEffect" || &id.sym == "useCallback" || &id.sym == "useMemo")
                        && call_expr.args.len() > 1
                    {
                        if let Some(ArrayLit { span: _, elems }) =
                            call_expr.args[1].expr.as_mut_array()
                        {
                            // remove `t` from deps
                            let deps = elems
                                .iter()
                                .filter(|&el| {
                                    let Some(el) = el else { return false };
                                    let Expr::Ident(id) = &*el.expr else { return true };
                                    // NOTE: using string but it's better to use ident instead
                                    &*id.sym != "t"
                                })
                                .cloned()
                                .collect();
                            call_expr.args[1].expr = Box::new(Expr::Array(ArrayLit {
                                span: DUMMY_SP,
                                elems: deps,
                            }))
                        };
                    }

                    if &id.sym == TRANSLATION_METHOD_NAME {
                        match call_expr.args.len() {
                            // t("foo", { bar: "a" })
                            n if n > 1 => {
                                if let Some(Lit::Str(lit)) = call_expr.args[0].expr.as_lit() {
                                    let (filename, path) =
                                        json_path_from_key(lit.value.to_string(), &self.scopes);
                                    if let Ok(value) = self.cache.get(filename, path) {
                                        let message = format!("`{}`", value)
                                            .replace("{{", "${")
                                            .replace("}}", "}");

                                        if let Some(ExprOrSpread {
                                            expr: id_or_obj,
                                            spread: _,
                                        }) = &call_expr.args.get(1)
                                        {
                                            if let Expr::Ident(ident) = &**id_or_obj {
                                                match self.make_interporation_node(
                                                    message,
                                                    ExprOrSpread {
                                                        spread: None,
                                                        expr: Box::new(Expr::Ident(ident.clone())),
                                                    },
                                                ) {
                                                    Ok(node) => {
                                                        *call_expr = node;
                                                    }
                                                    Err(_) => {
                                                        emit_error(
                                                            &format!(
                                                            "failed to make node from {} for {}",
                                                            value, lit.value
                                                        ),
                                                            lit.span,
                                                        );
                                                    }
                                                }
                                            } else if let Expr::Object(obj) = &**id_or_obj {
                                                match self.make_interporation_node(
                                                    message,
                                                    ExprOrSpread {
                                                        spread: None,
                                                        expr: Box::new(Expr::Object(obj.clone())),
                                                    },
                                                ) {
                                                    Ok(node) => {
                                                        *call_expr = node;
                                                    }
                                                    Err(_) => {
                                                        emit_error(
                                                            &format!(
                                                            "failed to make node from {} for {}",
                                                            value, lit.value
                                                        ),
                                                            lit.span,
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        emit_error(
                                            &format!("key: {} not found", lit.value),
                                            lit.span,
                                        );
                                    }
                                }
                            }
                            // t("foo")
                            1 => {
                                if let Some(Lit::Str(lit)) = call_expr.args[0].expr.as_mut_lit() {
                                    let (filename, path) =
                                        json_path_from_key(lit.value.to_string(), &self.scopes);
                                    if let Ok(value) = self.cache.get(filename, path) {
                                        let node = self.make_component_interporation_node(
                                            &value,
                                            &self.component_map,
                                        );
                                        if let Ok(node) = node {
                                            *n = *node;
                                        } else {
                                            emit_error(
                                                &format!(
                                                    "failed to make node from {} for {}",
                                                    value, lit.value
                                                ),
                                                lit.span,
                                            );
                                        }
                                    } else {
                                        emit_error(
                                            &format!("key: {} not found", lit.value),
                                            lit.span,
                                        );
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        n.visit_mut_children_with(self);

        if let Expr::JSXElement(jsx) = &mut *n {
            if let JSXElementName::Ident(ident) = &jsx.opening.name {
                if ident.sym.to_string() == TRANSLATION_COMPONENT_NAME {
                    *n = Expr::JSXFragment(JSXFragment {
                        span: DUMMY_SP,
                        opening: JSXOpeningFragment { span: DUMMY_SP },
                        children: jsx.children.clone(),
                        closing: JSXClosingFragment { span: DUMMY_SP },
                    });
                }
            }
        }
    }

    fn visit_mut_jsx_element_child(&mut self, n: &mut JSXElementChild) {
        n.visit_mut_children_with(self);
        if let JSXElementChild::JSXElement(jsx) = n {
            if let JSXElementName::Ident(ident) = &jsx.opening.name {
                if ident.sym.to_string() == TRANSLATION_COMPONENT_NAME {
                    *n = JSXElementChild::JSXFragment(JSXFragment {
                        span: DUMMY_SP,
                        opening: JSXOpeningFragment { span: DUMMY_SP },
                        children: jsx.children.clone(),
                        closing: JSXClosingFragment { span: DUMMY_SP },
                    });
                }
            }
        }
    }
}

impl TransformVisitor {
    pub fn make_component_interporation_node(
        &self,
        message: &str,
        map: &HashMap<String, JSXElement>,
    ) -> Result<Box<Expr>, &str> {
        let mut n = parse_code(format!("<>{}</>", message));
        if let Ok(node) = &mut n {
            if let Expr::JSXFragment(fragment) = &mut **node {
                let mut has_element = false;
                for child in &mut fragment.children {
                    if let JSXElementChild::JSXElement(c) = child {
                        has_element = true;
                        if let JSXElementName::Ident(ident) = &c.opening.name {
                            if let Some(value) = map.get(&ident.sym.to_string()) {
                                let children = c.children.clone();
                                let has_children = !children.is_empty();
                                let node = JSXElement {
                                    span: DUMMY_SP,
                                    opening: JSXOpeningElement {
                                        span: DUMMY_SP,
                                        self_closing: !has_children,
                                        ..value.opening.clone()
                                    },
                                    children,
                                    closing: if !has_children {
                                        None
                                    } else {
                                        Some(JSXClosingElement {
                                            span: DUMMY_SP,
                                            name: value.opening.name.clone(),
                                        })
                                    },
                                };
                                *child = JSXElementChild::JSXElement(Box::new(node));
                            }
                        }
                    }
                }
                if !has_element {
                    return Ok(Box::new(Expr::Lit(Lit::Str(Str {
                        span: DUMMY_SP,
                        value: JsWord::from(message),
                        raw: None,
                    }))));
                }
                if fragment.children.len() == 1 {
                    if let Some(JSXElementChild::JSXText(text)) = &fragment.children.get(0) {
                        return Ok(Box::new(Expr::Lit(Lit::Str(Str {
                            span: DUMMY_SP,
                            value: JsWord::from(text.value.to_string()),
                            raw: None,
                        }))));
                    }
                }
            }
            Ok(node.clone())
        } else {
            Err("invalid message format detected")
        }
    }

    pub fn make_interporation_node(
        &self,
        message: String,
        arg: ExprOrSpread,
    ) -> Result<CallExpr, &str> {
        let mut n = parse_code(message);
        if let Ok(tpl) = &mut n {
            if let Expr::Tpl(tpl) = &mut **tpl {
                for expr in &mut tpl.exprs {
                    if let Expr::Ident(ident) = &mut **expr {
                        ident.sym = JsWord::from(format!("v.{}", ident.sym));
                    }
                }
            }
            Ok(CallExpr {
                span: DUMMY_SP,
                callee: Callee::Expr(Box::new(Expr::Arrow(ArrowExpr {
                    span: DUMMY_SP,
                    params: vec![Pat::Ident(BindingIdent {
                        id: Ident {
                            span: DUMMY_SP,
                            sym: JsWord::from("v"),
                            optional: false,
                        },
                        type_ann: None,
                    })],
                    body: BlockStmtOrExpr::Expr(tpl.clone()),
                    type_params: None,
                    return_type: None,
                    is_async: false,
                    is_generator: false,
                }))),
                args: vec![arg],
                type_args: None,
            })
        } else {
            Err("todo")
        }
    }
}

#[plugin_transform]
pub fn process_transform(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let config = serde_json::from_str::<Config>(
        &metadata
            .get_transform_plugin_config()
            .expect("failed to get plugin config"),
    )
    .expect("invalid config");

    program.fold_with(&mut as_folder(TransformVisitor::new(config.base_dir)))
}

#[cfg(test)]
mod test;
