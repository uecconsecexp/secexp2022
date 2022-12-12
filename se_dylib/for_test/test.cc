#include <iostream>
#include <thread>
#include <chrono>

extern "C"
{
    void *new_server();
    int s_send(void *server, const char *data, int len);
    int s_receive(void *server, char *data, int len);
    int s_send_table(void *server, const double *data, int row_num, int col_num);
    int s_receive_table(void *server, double *data, int len, int *row_num, int *col_num);
    void s_close(void *server);

    void *new_client(const char *server_address);
    int c_send(void *client, const char *data, int len);
    int c_receive(void *client, char *data, int len);
    int c_send_table(void *client, const double *data, int row_num, int col_num);
    int c_receive_table(void *client, double *data, int len, int *row_num, int *col_num);
    void c_close(void *client);
}

void server()
{
    void *server = new_server();
    char data[1024];
    int len = s_receive(server, data, 1024);

    if (len < 0)
    {
        std::cerr << "Error receiving data" << std::endl;
        return;
    }

    std::cout << "Received: " << data << std::endl;

    int res = s_send(server, data, len);

    if (res != 0)
    {
        std::cerr << "Error sending data" << std::endl;
        return;
    }

    double *table = new double[6];
    int row, col;
    res = s_receive_table(server, table, 6, &row, &col);

    if (res != 0)
    {
        std::cerr << "Error receiving table" << std::endl;
        return;
    }

    res = s_send_table(server, table, row, col);

    if (res != 0)
    {
        std::cerr << "Error sending table" << std::endl;
        return;
    }

    s_close(server);
}

void client(const char *server_address)
{
    void *client = new_client(server_address);
    int res = c_send(client, "Hello", 5);

    if (res != 0)
    {
        std::cerr << "Error sending data" << std::endl;
        return;
    }

    char data[1024];
    int len = c_receive(client, data, 1024);

    if (len < 0)
    {
        std::cerr << "Error receiving data" << std::endl;
        return;
    }

    std::cout << "Received: " << data << std::endl;

    double table[6] = {1, 2, 3, 4, 5, 6};
    int row = 2;
    int col = 3;
    res = c_send_table(client, table, row, col);

    if (res != 0)
    {
        std::cerr << "Error sending table" << std::endl;
        return;
    }

    double *table2 = new double[6];
    int row2, col2;
    res = c_receive_table(client, table2, 6, &row2, &col2);

    if (res != 0)
    {
        std::cerr << "Error receiving table" << std::endl;
        return;
    }

    for (int i = 0; i < row2; i++)
    {
        for (int j = 0; j < col2; j++)
        {
            if (table[i * col2 + j] != table2[i * col2 + j])
            {
                std::cerr << "Error: table2[" << i << "][" << j << "] = " << table2[i * col2 + j] << " != " << table[i * col2 + j] << std::endl;
                return;
            }
        }
    }

    std::cout << "Received valid table" << std::endl;

    c_close(client);
}

int main()
{
    std::thread server_thread(server);
    std::this_thread::sleep_for(std::chrono::milliseconds(1000));
    std::thread client_thread(client, "0.0.0.0");

    server_thread.join();
    client_thread.join();
}