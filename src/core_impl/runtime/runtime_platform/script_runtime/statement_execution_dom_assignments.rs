use super::*;

#[path = "statement_execution_dom_assign_element_props.rs"]
mod statement_execution_dom_assign_element_props;
#[path = "statement_execution_dom_assign_form_props.rs"]
mod statement_execution_dom_assign_form_props;
#[path = "statement_execution_dom_assign_media_props.rs"]
mod statement_execution_dom_assign_media_props;
#[path = "statement_execution_dom_assign_navigation_anchor_props.rs"]
mod statement_execution_dom_assign_navigation_anchor_props;

impl Harness {
    fn execute_dom_assign_stmt(
        &mut self,
        target: &DomQuery,
        prop: &DomProp,
        op: &VarAssignOp,
        expr: &Expr,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<()> {
        if matches!(
            op,
            VarAssignOp::LogicalAnd | VarAssignOp::LogicalOr | VarAssignOp::Nullish
        ) {
            let previous = self.eval_expr(
                &Expr::DomRead {
                    target: target.clone(),
                    prop: prop.clone(),
                },
                env,
                event_param,
                event,
            )?;
            let should_assign = match op {
                VarAssignOp::LogicalAnd => previous.truthy(),
                VarAssignOp::LogicalOr => !previous.truthy(),
                VarAssignOp::Nullish => {
                    matches!(&previous, Value::Null | Value::Undefined)
                }
                _ => true,
            };
            if !should_assign {
                return Ok(());
            }
        }

        let value = self.eval_expr(expr, env, event_param, event)?;
        if let Some(key) = Self::object_key_from_dom_prop(prop) {
            let receiver = match target {
                DomQuery::Var(name) => env.get(name).cloned(),
                DomQuery::VarPath { .. } | DomQuery::Index { .. } => {
                    self.resolve_dom_query_value_runtime(target, env)?
                }
                _ => None,
            };
            if let Some(receiver) = receiver {
                if !matches!(receiver, Value::Node(_) | Value::NodeList(_)) {
                    let assignment_target = match target {
                        DomQuery::Var(name) => name.clone(),
                        _ => target.describe_call(),
                    };
                    let mut assignment_value = value.clone();
                    let is_event_param_target = match target {
                        DomQuery::Var(name) => {
                            event_param.as_ref().is_some_and(|param| param == name)
                        }
                        _ => false,
                    };
                    if key == "returnValue"
                        && is_event_param_target
                        && (event.before_unload_interface
                            || event.event_type.eq_ignore_ascii_case("beforeunload"))
                    {
                        let return_value = assignment_value.as_string();
                        event.before_unload_interface = true;
                        event.before_unload_return_value = return_value.clone();
                        if event.cancelable && !return_value.is_empty() {
                            event.default_prevented = true;
                        }
                        assignment_value = Value::String(return_value);
                    }
                    self.set_object_assignment_property(
                        &receiver,
                        &Value::String(key.to_string()),
                        assignment_value,
                        &assignment_target,
                        env,
                        event,
                    )?;
                    return Ok(());
                }
            }
        }

        let node = self.resolve_dom_query_required_runtime(target, env)?;
        if self.try_execute_dom_assign_form_prop(node, prop, &value, env, event)? {
            return Ok(());
        }
        if self.try_execute_dom_assign_element_prop(node, prop, &value, event)? {
            return Ok(());
        }
        if self.try_execute_dom_assign_media_prop(node, prop, &value, event)? {
            return Ok(());
        }
        if self.try_execute_dom_assign_navigation_anchor_prop(node, prop, &value, event)? {
            return Ok(());
        }

        match prop {
            DomProp::Attributes
            | DomProp::AssignedSlot
            | DomProp::FilesLength
            | DomProp::ValidationMessage
            | DomProp::Validity
            | DomProp::ValidityValueMissing
            | DomProp::ValidityTypeMismatch
            | DomProp::ValidityPatternMismatch
            | DomProp::ValidityTooLong
            | DomProp::ValidityTooShort
            | DomProp::ValidityRangeUnderflow
            | DomProp::ValidityRangeOverflow
            | DomProp::ValidityStepMismatch
            | DomProp::ValidityBadInput
            | DomProp::ValidityValid
            | DomProp::ValidityCustomError
            | DomProp::NodeType
            | DomProp::ClassListLength
            | DomProp::PartLength
            | DomProp::TagName
            | DomProp::LocalName
            | DomProp::NamespaceUri
            | DomProp::Prefix
            | DomProp::NextElementSibling
            | DomProp::PreviousElementSibling
            | DomProp::ClientWidth
            | DomProp::ClientHeight
            | DomProp::ClientLeft
            | DomProp::ClientTop
            | DomProp::CurrentCssZoom
            | DomProp::ScrollLeftMax
            | DomProp::ScrollTopMax
            | DomProp::ShadowRoot
            | DomProp::AriaElementRefSingle(_)
            | DomProp::AriaElementRefList(_)
            | DomProp::OffsetWidth
            | DomProp::ValueLength
            | DomProp::OffsetHeight
            | DomProp::OffsetLeft
            | DomProp::OffsetTop
            | DomProp::ScrollWidth
            | DomProp::ScrollHeight
            | DomProp::ScrollLeft
            | DomProp::ScrollTop
            | DomProp::ActiveElement
            | DomProp::ActiveViewTransition
            | DomProp::AdoptedStyleSheetsLength
            | DomProp::CharacterSet
            | DomProp::CompatMode
            | DomProp::ContentType
            | DomProp::ReadyState
            | DomProp::Referrer
            | DomProp::Url
            | DomProp::DocumentUri
            | DomProp::BaseUri
            | DomProp::LocationOrigin
            | DomProp::LocationAncestorOrigins
            | DomProp::History
            | DomProp::HistoryLength
            | DomProp::HistoryState
            | DomProp::DefaultView
            | DomProp::VisibilityState
            | DomProp::Forms
            | DomProp::Images
            | DomProp::Links
            | DomProp::Scripts
            | DomProp::Children
            | DomProp::ChildElementCount
            | DomProp::FirstElementChild
            | DomProp::LastElementChild
            | DomProp::CurrentScript
            | DomProp::FormsLength
            | DomProp::ImagesLength
            | DomProp::LinksLength
            | DomProp::ScriptsLength
            | DomProp::ChildrenLength
            | DomProp::AnchorOrigin
            | DomProp::AnchorRelList
            | DomProp::AnchorRelListLength => {
                let call = self.describe_dom_prop(prop);
                return Err(Error::ScriptRuntime(format!("{call} is read-only")));
            }
            _ => {}
        }

        Ok(())
    }

    pub(crate) fn try_execute_dom_assign_stmt(
        &mut self,
        stmt: &Stmt,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<Option<ExecFlow>> {
        let Stmt::DomAssign {
            target,
            prop,
            op,
            expr,
        } = stmt
        else {
            return Ok(None);
        };

        self.execute_dom_assign_stmt(target, prop, op, expr, env, event_param, event)?;
        Ok(Some(ExecFlow::Continue))
    }
}
