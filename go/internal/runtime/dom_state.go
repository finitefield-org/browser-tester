package runtime

func (s *Session) DOMReady() bool {
	if s == nil {
		return false
	}
	return s.domReady && s.domErr == nil
}

func (s *Session) DOMError() string {
	if s == nil || s.domErr == nil {
		return ""
	}
	return s.domErr.Error()
}
