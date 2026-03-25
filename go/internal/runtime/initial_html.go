package runtime

func (s *Session) InitialHTML() string {
	if s == nil {
		return ""
	}
	return s.config.HTML
}
