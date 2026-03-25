package runtime

func (s *Session) DfnCount() int {
	return s.elementCountForSelector("dfn")
}
