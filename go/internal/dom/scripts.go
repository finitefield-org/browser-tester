package dom

import "fmt"

func (s *Store) Scripts() (HTMLCollection, error) {
	if s == nil {
		return HTMLCollection{}, fmt.Errorf("dom store is nil")
	}
	return newScriptCollection(s, s.documentID), nil
}

func (s *Store) Images() (HTMLCollection, error) {
	if s == nil {
		return HTMLCollection{}, fmt.Errorf("dom store is nil")
	}
	return newImageCollection(s, s.documentID), nil
}

func (s *Store) Forms() (HTMLCollection, error) {
	if s == nil {
		return HTMLCollection{}, fmt.Errorf("dom store is nil")
	}
	return newFormCollection(s, s.documentID), nil
}

func (s *Store) FormElements(formID NodeID) (HTMLCollection, error) {
	if s == nil {
		return HTMLCollection{}, fmt.Errorf("dom store is nil")
	}
	return newFormElementsCollection(s, formID), nil
}

func (s *Store) SelectedOptions(selectID NodeID) (HTMLCollection, error) {
	if s == nil {
		return HTMLCollection{}, fmt.Errorf("dom store is nil")
	}
	return newSelectedOptionsCollection(s, selectID), nil
}

func (s *Store) Options(selectID NodeID) (HTMLCollection, error) {
	if s == nil {
		return HTMLCollection{}, fmt.Errorf("dom store is nil")
	}
	return newOptionsCollection(s, selectID), nil
}

func (s *Store) DatalistOptions(datalistID NodeID) (HTMLCollection, error) {
	if s == nil {
		return HTMLCollection{}, fmt.Errorf("dom store is nil")
	}
	return newOptionsCollection(s, datalistID), nil
}

func (s *Store) FieldsetElements(fieldsetID NodeID) (HTMLCollection, error) {
	if s == nil {
		return HTMLCollection{}, fmt.Errorf("dom store is nil")
	}
	return newFormElementsCollection(s, fieldsetID), nil
}

func (s *Store) Cells(rowID NodeID) (HTMLCollection, error) {
	if s == nil {
		return HTMLCollection{}, fmt.Errorf("dom store is nil")
	}
	return newTableCellsCollection(s, rowID), nil
}

func (s *Store) TBodies(tableID NodeID) (HTMLCollection, error) {
	if s == nil {
		return HTMLCollection{}, fmt.Errorf("dom store is nil")
	}
	return newTableBodiesCollection(s, tableID), nil
}

func (s *Store) Rows(tableID NodeID) (HTMLCollection, error) {
	if s == nil {
		return HTMLCollection{}, fmt.Errorf("dom store is nil")
	}
	return newRowsCollection(s, tableID), nil
}

func (s *Store) Links() (HTMLCollection, error) {
	if s == nil {
		return HTMLCollection{}, fmt.Errorf("dom store is nil")
	}
	return newLinkCollection(s, s.documentID), nil
}

func (s *Store) Anchors() (HTMLCollection, error) {
	if s == nil {
		return HTMLCollection{}, fmt.Errorf("dom store is nil")
	}
	return newAnchorCollection(s, s.documentID), nil
}
