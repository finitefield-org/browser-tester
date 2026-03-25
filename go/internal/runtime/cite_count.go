package runtime

func (s *Session) CiteCount() int {
	return s.elementCountForSelector("cite")
}
