package browsertester

import dom "browsertester/internal/dom"

// ClassListView is a live view over an element's class attribute.
type ClassListView struct {
	list dom.ClassList
}

func (v ClassListView) Values() []string {
	return v.list.Values()
}

func (v ClassListView) Contains(token string) bool {
	return v.list.Contains(token)
}

func (v ClassListView) Add(tokens ...string) error {
	if v.list == (dom.ClassList{}) {
		return NewError(ErrorKindDOM, "class list is unavailable")
	}
	if err := v.list.Add(tokens...); err != nil {
		return NewError(ErrorKindDOM, err.Error())
	}
	return nil
}

func (v ClassListView) Remove(tokens ...string) error {
	if v.list == (dom.ClassList{}) {
		return NewError(ErrorKindDOM, "class list is unavailable")
	}
	if err := v.list.Remove(tokens...); err != nil {
		return NewError(ErrorKindDOM, err.Error())
	}
	return nil
}

// DatasetView is a live view over an element's dataset.
type DatasetView struct {
	dataset dom.Dataset
}

func (v DatasetView) Values() map[string]string {
	return v.dataset.Values()
}

func (v DatasetView) Get(name string) (string, bool) {
	return v.dataset.Get(name)
}

func (v DatasetView) Set(name, value string) error {
	if v.dataset == (dom.Dataset{}) {
		return NewError(ErrorKindDOM, "dataset is unavailable")
	}
	if err := v.dataset.Set(name, value); err != nil {
		return NewError(ErrorKindDOM, err.Error())
	}
	return nil
}

func (v DatasetView) Remove(name string) error {
	if v.dataset == (dom.Dataset{}) {
		return NewError(ErrorKindDOM, "dataset is unavailable")
	}
	if err := v.dataset.Remove(name); err != nil {
		return NewError(ErrorKindDOM, err.Error())
	}
	return nil
}

func (h *Harness) ClassList(selector string) (ClassListView, error) {
	if h == nil || h.session == nil {
		return ClassListView{}, NewError(ErrorKindDOM, "class list is unavailable")
	}
	list, err := h.session.ClassList(selector)
	if err != nil {
		return ClassListView{}, NewError(ErrorKindDOM, err.Error())
	}
	return ClassListView{list: list}, nil
}

func (h *Harness) Dataset(selector string) (DatasetView, error) {
	if h == nil || h.session == nil {
		return DatasetView{}, NewError(ErrorKindDOM, "dataset is unavailable")
	}
	dataset, err := h.session.Dataset(selector)
	if err != nil {
		return DatasetView{}, NewError(ErrorKindDOM, err.Error())
	}
	return DatasetView{dataset: dataset}, nil
}
