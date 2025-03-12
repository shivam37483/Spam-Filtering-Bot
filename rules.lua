function check_spam(message)
    local score = 0
    if string.lower(message):find("spam") then
        score = score + 10
    end
    if string.lower(message):find("http") then
        score = score + 5
    end
    return score
end