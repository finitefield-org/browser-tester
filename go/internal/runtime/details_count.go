package runtime

func (s *Session) DetailsCount() int {
	return s.elementCountForSelector("details")
}
