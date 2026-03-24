package browsertester

func (h *Harness) InnerHTML(selector string) (string, error) {
	if h == nil || h.session == nil {
		return "", NewError(ErrorKindDOM, "inner html is unavailable")
	}
	value, err := h.session.InnerHTML(selector)
	if err != nil {
		return "", NewError(ErrorKindDOM, err.Error())
	}
	return value, nil
}

func (h *Harness) OuterHTML(selector string) (string, error) {
	if h == nil || h.session == nil {
		return "", NewError(ErrorKindDOM, "outer html is unavailable")
	}
	value, err := h.session.OuterHTML(selector)
	if err != nil {
		return "", NewError(ErrorKindDOM, err.Error())
	}
	return value, nil
}

func (h *Harness) SetInnerHTML(selector, markup string) error {
	if h == nil || h.session == nil {
		return NewError(ErrorKindDOM, "set inner html is unavailable")
	}
	if err := h.session.SetInnerHTML(selector, markup); err != nil {
		return NewError(ErrorKindDOM, err.Error())
	}
	return nil
}

func (h *Harness) SetOuterHTML(selector, markup string) error {
	if h == nil || h.session == nil {
		return NewError(ErrorKindDOM, "set outer html is unavailable")
	}
	if err := h.session.SetOuterHTML(selector, markup); err != nil {
		return NewError(ErrorKindDOM, err.Error())
	}
	return nil
}

func (h *Harness) InsertAdjacentHTML(selector, position, markup string) error {
	if h == nil || h.session == nil {
		return NewError(ErrorKindDOM, "insert adjacent html is unavailable")
	}
	if err := h.session.InsertAdjacentHTML(selector, position, markup); err != nil {
		return NewError(ErrorKindDOM, err.Error())
	}
	return nil
}

func (h *Harness) RemoveNode(selector string) error {
	if h == nil || h.session == nil {
		return NewError(ErrorKindDOM, "remove node is unavailable")
	}
	if err := h.session.RemoveNode(selector); err != nil {
		return NewError(ErrorKindDOM, err.Error())
	}
	return nil
}
