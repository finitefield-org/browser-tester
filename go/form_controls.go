package browsertester

func (h *Harness) TypeText(selector, text string) error {
	if h == nil || h.session == nil {
		return NewError(ErrorKindDOM, "type text is unavailable")
	}
	if err := h.session.TypeText(selector, text); err != nil {
		return NewError(ErrorKindDOM, err.Error())
	}
	return nil
}

func (h *Harness) SetChecked(selector string, checked bool) error {
	if h == nil || h.session == nil {
		return NewError(ErrorKindDOM, "set checked is unavailable")
	}
	if err := h.session.SetChecked(selector, checked); err != nil {
		return NewError(ErrorKindDOM, err.Error())
	}
	return nil
}

func (h *Harness) SetSelectValue(selector, value string) error {
	if h == nil || h.session == nil {
		return NewError(ErrorKindDOM, "set select value is unavailable")
	}
	if err := h.session.SetSelectValue(selector, value); err != nil {
		return NewError(ErrorKindDOM, err.Error())
	}
	return nil
}

func (h *Harness) Submit(selector string) error {
	if h == nil || h.session == nil {
		return NewError(ErrorKindDOM, "submit is unavailable")
	}
	if err := h.session.Submit(selector); err != nil {
		return NewError(ErrorKindDOM, err.Error())
	}
	return nil
}
