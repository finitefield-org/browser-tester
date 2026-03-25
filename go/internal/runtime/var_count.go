package runtime

func (s *Session) VarCount() int {
	return s.elementCountForSelector("var")
}
