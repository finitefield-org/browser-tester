package browsertester

type InteractionKind string

const (
	InteractionKindClick          InteractionKind = "click"
	InteractionKindFocus          InteractionKind = "focus"
	InteractionKindBlur           InteractionKind = "blur"
	InteractionKindTypeText       InteractionKind = "type_text"
	InteractionKindSetChecked     InteractionKind = "set_checked"
	InteractionKindSetSelectValue InteractionKind = "set_select_value"
	InteractionKindSubmit         InteractionKind = "submit"
)

type Interaction struct {
	Kind     InteractionKind
	Selector string
}
