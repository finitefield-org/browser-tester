package runtime

func (s *Session) SummaryCount() int {
	return s.elementCountForSelector("summary")
}
