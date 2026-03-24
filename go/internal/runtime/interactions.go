package runtime

type InteractionKind string

const (
	InteractionKindClick          InteractionKind = "click"
	InteractionKindTypeText       InteractionKind = "type_text"
	InteractionKindSetChecked     InteractionKind = "set_checked"
	InteractionKindSetSelectValue InteractionKind = "set_select_value"
	InteractionKindSubmit         InteractionKind = "submit"
	InteractionKindFocus          InteractionKind = "focus"
	InteractionKindBlur           InteractionKind = "blur"
)

type Interaction struct {
	Kind     InteractionKind
	Selector string
}
