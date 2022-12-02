package connector

import (
	"encoding/json"
	"fmt"
	"strconv"
)

type matrix struct {
	Data [][]string `json:"data"`
}

func newMatrix(table [][]float64) (matrix, error) {
	rows := len(table)
	if rows < 1 {
		return matrix{}, fmt.Errorf("table must have at least one row")
	}
	cols := len(table[0])

	str_table := make([][]string, rows)
	for i, row := range table {
		str_table[i] = make([]string, cols)
		for j, n := range row {
			if len(table[i]) != cols {
				return matrix{}, fmt.Errorf("table must be a rectangle")
			}

			str_table[i][j] = strconv.FormatFloat(n, 'E', -1, 64)
		}
	}

	return matrix{str_table}, nil
}

func strMatrix2FloatMatrix(table [][]string) ([][]float64, error) {
	rows := len(table)
	if rows < 1 {
		return nil, fmt.Errorf("table must have at least one row")
	}
	cols := len(table[0])

	float_table := make([][]float64, rows)
	for i, row := range table {
		float_table[i] = make([]float64, cols)
		for j, s := range row {
			if len(table[i]) != cols {
				return nil, fmt.Errorf("table must be a rectangle")
			}

			n, err := strconv.ParseFloat(s, 64)
			if err != nil {
				return nil, err
			}
			float_table[i][j] = n
		}
	}

	return float_table, nil
}

func sendTable(communicator Communicator, table [][]float64) error {
	mtrx, err := newMatrix(table)
	if err != nil {
		return err
	}

	data, err := json.Marshal(mtrx)
	if err != nil {
		return err
	}

	return send(communicator, data)
}

func receiveTable(communicator Communicator) ([][]float64, error) {
	data, err := receive(communicator)
	if err != nil {
		return nil, err
	}

	var matrix matrix
	if err := json.Unmarshal(data, &matrix); err != nil {
		return nil, err
	}

	return strMatrix2FloatMatrix(matrix.Data)
}

func (chugaku ChugakuClient) SendTable(table [][]float64) error {
	return sendTable(chugaku, table)
}

func (chugaku ChugakuClient) ReceiveTable() ([][]float64, error) {
	return receiveTable(chugaku)
}

func (yobikou YobikouServer) SendTable(table [][]float64) error {
	return sendTable(yobikou, table)
}

func (yobikou YobikouServer) ReceiveTable() ([][]float64, error) {
	return receiveTable(yobikou)
}
