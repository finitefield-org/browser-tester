package runtime

func (s *Session) documentCurrentScript() string {
	if s == nil {
		return ""
	}
	return s.currentScriptHTML
}
