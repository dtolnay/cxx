#pragma once
#include <string>
#include <iostream>
#include <memory>

struct Person
{
    std::string name;

    Person()
    {
        this->name = "cpp expert!";
    }

    void print_name()
    {
        std::cout << this->name << std::endl;
    }
};

const std::string &get_name(const Person &person)
{
    return person.name;
}

std::unique_ptr<Person> make_person()
{
    return std::make_unique<Person>();
}
