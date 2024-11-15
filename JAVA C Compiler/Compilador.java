import java.io.BufferedReader;
import java.io.FileReader;
import java.io.IOException;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

public class Compilador {
    // Expresiones regulares
    private static final String IDENTIFIER_REGEX = "[a-zA-Z_][a-zA-Z0-9_]{0,30}";
    private static final String NUMBER_REGEX = "[0-9]+";
    private static final String ASSIGNMENT_REGEX = "=";
    private static final String LEFT_PAREN_REGEX = "\\(";
    private static final String RIGHT_PAREN_REGEX = "\\)";
    private static final String LEFT_BRACE_REGEX = "\\{";
    private static final String RIGHT_BRACE_REGEX = "\\}";
    private static final String LEFT_BRACKET_REGEX = "\\[";
    private static final String RIGHT_BRACKET_REGEX = "\\]";
    private static final String COMA_REGEX = ",";
    private static final String DOT_REGEX = ".";
    private static final String SEMICOLON_REGEX = ";";
    private static final String COLON_REGEX = ":";
    private static final String STRING_REGEX = "\"(([^\"\\\\]|\\\\.)*)\""; // Soporte para cadenas
    private static final String CHAR_REGEX = "'([^'\\\\]|\\\\.){1}'"; // Soporte para un solo caracter
    private static final String SINGLE_LINE_COMMENT_REGEX = "\\/\\/.*"; // Soporte para comentarios unilinea
    private static final String MULTI_LINE_COMMENT_REGEX = "/\\*([^*]|\\*(?!/))*\\*/"; // Soporte para comentarios multilinea
    private static final String COMBINED_ASSIGNMENT_REGEX = "(\\+=|-=|\\*=|/=|\\+\\+|--)";
    private static final String OPERATOR_REGEX = "[+\\-*/%^!]+";
    private static final String LOGIC_OPERATOR_REGEX = "(&&|\\|\\|)";
    private static final String BIT_OPERATOR_REGEX = "(&|\\||\\^|~|<<|>>)";
    private static final String COMPARATOR_REGEX = "(==|!=|<|<=|>|>=|<>)"; // Ajustado para comparadores
    private static final String RESERVED_WORDS_REGEX = "(auto|else|long|switch|break|enum|register|typedef|case|extern|return|union|char|float|short|unsigned|const|for|signed|void|continue|goto|sizeof|volatile|default|if|static|while|do|int|struct|_Packed|double)";
    private static final String DATA_TYPE_REGEX = "(int|char|float|double|void)";
    private static final String WHILE_REGEX = "while";
    private static final String IF_REGEX = "if";
    private static final String CASE_REGEX = "case";
    private static final String DO_REGEX = "do";

    // ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

    private static final String INITIALIZE_LINE_REGEX = "\\|DT\\|\\|ID\\|((\\|AS\\|(\\|ID\\||\\|NU\\|)){0,1}(\\|OP\\|(\\|ID\\||\\|NU\\|))*)(\\|COMA\\|\\|ID\\|((\\|AS\\|(\\|ID\\||\\|NU\\|)){0,1}(\\|OP\\|(\\|ID\\||\\|NU\\|))*))*\\|SC\\|";
    private static final String ASSIGN_LINE_REGEX = "\\|ID\\|((\\|AS\\|(\\|ID\\||\\|NU\\|)){1}(\\|OP\\|(\\|ID\\||\\|NU\\|))*)\\|SC\\|";
    private static final String FUNCTION_LINE_REGEX = "\\|DT\\|\\|ID\\|\\|LP\\|(\\|DT\\|\\|ID\\|){0,1}(\\|COMA\\|\\|DT\\|\\|ID\\|)*\\|RP\\|\\|LB\\|";
    private static final String IF_LINE_REGEX = "\\|IF\\|\\|LP\\|((\\|ID\\||\\|NU\\|)\\|CO\\|(\\|ID\\||\\|NU\\|)){1}(\\|LO\\|(\\|ID\\||\\|NU\\|)\\|CO\\|(\\|ID\\||\\|NU\\|))*\\|RP\\|\\|LB\\|";
    private static final String WHILE_LINE_REGEX = "\\|WH\\|\\|LP\\|((\\|ID\\||\\|NU\\|)\\|CO\\|(\\|ID\\||\\|NU\\|)){1}(\\|LO\\|(\\|ID\\||\\|NU\\|)\\|CO\\|(\\|ID\\||\\|NU\\|))*\\|RP\\|\\|LB\\|";
    private static final String END_LINE_REGEX = "\\|RB\\|";
    private static final String DO_LINE_REGEX = "\\|DO\\|\\|LB\\|";
    private static final String DO_END_REGEX = "\\|WH\\|\\|LP\\|((\\|ID\\||\\|NU\\|)\\|CO\\|(\\|ID\\||\\|NU\\|)){1}(\\|LO\\|(\\|ID\\||\\|NU\\|)\\|CO\\|(\\|ID\\||\\|NU\\|))*\\|RP\\|\\|SC\\|";
    private static final String SEMICOLON_LINE_REGEX = "\\|SC\\|";

    public static void main(String[] args) {
        // Ruta del archivo .txt
        String filePath = "prueba.txt";

        try (BufferedReader br = new BufferedReader(new FileReader(filePath))) {

            StringBuilder contentBuilder = new StringBuilder();
            String line;

            while ((line = br.readLine()) != null) {
                line = handleComments(line);
                contentBuilder.append(line).append(" "); // Unir las líneas en una sola
            }

            String content = contentBuilder.toString();
            String[] logicalLines = content.split("(?<=[;{}])");// Divide cada línea lógica por ';', aperturas y cerraduras

            int linea = 1;

            boolean esValido = false;

            for (String logicalLine : logicalLines) {

                esValido = false;
                String errorCon;

                logicalLine = logicalLine.trim(); // Quitar espacios
                if (!logicalLine.isEmpty()) {

                    String primeLine = logicalLine;

                    String auxLine = handleComments(logicalLine);
                    logicalLine = auxLine;

                    // Aplica la función que maneja cadenas y caracteres
                    auxLine = handleStringsAndChars(logicalLine);
                    logicalLine = auxLine;

                    logicalLine = logicalLine.replace("&&", "TEMPAND");
                    logicalLine = logicalLine.replace("||", "TEMPOR");
                    logicalLine = logicalLine.replace("==", "TEMPEQUALS");
                    logicalLine = logicalLine.replace(">=", "TEMPEQG");
                    logicalLine = logicalLine.replace("<=", "TEMPEQL");
                    logicalLine = logicalLine.replace("!=", "TEMPNOT");
                    logicalLine = logicalLine.replace("<>", "TEMPDIFF");

                    // Paso 2: Tokeniza la línea normalmente
                    String[] tokens = logicalLine.split("(?<=\\W)|(?=\\W)"); // Separar por símbolos no alfanuméricos

                    for (int i = 0; i < tokens.length; i++) {
                        switch (tokens[i]) {
                            case "TEMPAND":
                                tokens[i] = "&&";
                                break;
                            case "TEMPOR":
                                tokens[i] = "||";
                                break;
                            case "TEMPEQUALS":
                                tokens[i] = "==";
                                break;
                            case "TEMPEQG":
                                tokens[i] = ">=";
                                break;
                            case "TEMPEQL":
                                tokens[i] = "<=";
                                break;
                            case "TEMPNOT":
                                tokens[i] = "!=";
                                break;
                            case "TEMPDIFF":
                                tokens[i] = "<>";
                                break;
                            default:
                                break;
                        }
                    }

                    System.out.println("");
                    System.out.println("Analizando línea: " + primeLine);
                    System.out.print(">>");

                    StringBuffer newLine = new StringBuffer();

                    for (String token : tokens) {
                        token = token.trim();
                        if (token.isEmpty())
                            continue; // Ignorar tokens vacíos
                        if (isDataType(token)) {
                            newLine.append("|DT|");
                            esValido = true;
                        } else if (isIf(token)) {
                            newLine.append("|IF|");
                            esValido = true;
                        } else if (isDo(token)) {
                            newLine.append("|DO|");
                            esValido = true;
                        } else if (isWhile(token)) {
                            newLine.append("|WH|");
                            esValido = true;
                        } else if (isReservedWord(token)) {
                            newLine.append("|PR|");
                            esValido = true;
                        } else if (isString(token)) {
                            newLine.append("|STR|");
                            esValido = true;
                        } else if (isChar(token)) {
                            newLine.append("|CHR|"); // Token reconocido como carácter
                            esValido = true;
                        } else if (isIdentifier(token)) {
                            newLine.append("|ID|");
                            esValido = true;
                        } else if (isCombinedAssignment(token)) {
                            newLine.append("|AC|");
                            esValido = true;
                        } else if (isAssignment(token)) {
                            newLine.append("|AS|");
                            esValido = true;
                        } else if (isDigit(token)) {
                            newLine.append("|NU|");
                            esValido = true;
                        } else if (isOperator(token)) {
                            newLine.append("|OP|");
                            esValido = true;
                        } else if (isLogicOperator(token)) {
                            newLine.append("|LO|");
                            esValido = true;
                        } else if (isBitOperator(token)) {
                            newLine.append("|BO|");
                            esValido = true;
                        } else if (isComparator(token)) {
                            newLine.append("|CO|");
                            esValido = true;
                        } else if (isLeftParen(token)) {
                            newLine.append("|LP|");
                            esValido = true;
                        } else if (isRightParen(token)) {
                            newLine.append("|RP|");
                            esValido = true;
                        } else if (isLeftBrace(token)) {
                            newLine.append("|LB|");
                            esValido = true;
                        } else if (isRightBrace(token)) {
                            newLine.append("|RB|");
                            esValido = true;
                        } else if (isLeftBracket(token)) {
                            newLine.append("|LBR|");
                            esValido = true;
                        } else if (isRightBracket(token)) {
                            newLine.append("|RBR|");
                            esValido = true;
                        } else if (isComa(token)) {
                            newLine.append("|COMA|");
                            esValido = true;
                        } else if (isColon(token)) {
                            newLine.append("|COL|");
                            esValido = true;
                        } else if (isSemicolon(token)) {
                            newLine.append("|SC|");
                            esValido = true;
                        } else if (isDot(token)) {
                            newLine.append("|DOT|");
                            esValido = true;
                        } else {
                            errorCon = "Token no reconocido: " + token;
                            throw new compilerError("|ERROR|" + errorCon + "En Linea:" + linea);
                        }
                    }

                    esValido = false;

                    if (isSemiColonLine(newLine.toString())) {
                        System.out.println("|WARNING| Linea" + linea +" vacia.");
                        esValido = true;
                    } else if (isInitialLine(newLine.toString())) {
                        esValido = true;
                    } else if (isAssignLine(newLine.toString())) {
                        esValido = true;
                    } else if (isFunctionLine(newLine.toString())) {
                        esValido = true;
                    } else if (isIfLine(newLine.toString())) {
                        esValido = true;
                    } else if (isWhileLine(newLine.toString())) {
                        esValido = true;
                    } else if (isEndLine(newLine.toString())) {
                        esValido = true;
                    } else if (isDoLine(newLine.toString())) {
                        esValido = true;
                    } else if (isDoEndLine(newLine.toString())) {
                        esValido = true;
                    } else {
                        throw new compilerError("|ERROR| Sintaxis erronea. En Linea:" + linea);
                    }

                    if(esValido)
                        System.out.print("Linea Valida");
                    else
                        throw new compilerError("|ERROR| Error desconocido. En Linea:" + linea);

                } else {
                    esValido = true;
                }

                linea++;

            }
            if(esValido)
                System.out.println("Compilado Exitoso");
            else
                throw new compilerError("|ERROR| Error desconocido en archivo.");
                

        } catch (IOException e) {
            e.getMessage();
            e.printStackTrace();
        } catch (compilerError e) {
            System.out.println(e.getMessage());
            //e.printStackTrace();
        }

        System.out.println("FIN DEL PROGRAMA");

    }

    // Paso 1: Reemplaza las cadenas y caracteres por marcadores
    public static String handleStringsAndChars(String line) {

        String newLine = line;
        Pattern stringPattern = Pattern.compile(STRING_REGEX);
        Pattern charPattern = Pattern.compile(CHAR_REGEX);

        // Reemplazar cadenas por "STR"
        Matcher stringMatcher = stringPattern.matcher(newLine);
        newLine = stringMatcher.replaceAll(" STR ");

        // Reemplazar caracteres por "CHR"
        Matcher charMatcher = charPattern.matcher(newLine);
        newLine = charMatcher.replaceAll(" CHR ");

        return newLine;
    }

    // Función para manejar comentarios
    public static String handleComments(String line) {

        String newLine = line;
        Pattern singleLinePattern = Pattern.compile(SINGLE_LINE_COMMENT_REGEX);
        Pattern multiLinePattern = Pattern.compile(MULTI_LINE_COMMENT_REGEX);

        // Reemplazar comentarios de una línea por "COMMENT"
        Matcher singleLineMatcher = singleLinePattern.matcher(newLine);
        newLine = singleLineMatcher.replaceAll("");

        // Reemplazar comentarios de varias líneas por "COMMENT"
        Matcher multiLineMatcher = multiLinePattern.matcher(newLine);
        line = multiLineMatcher.replaceAll("");

        return line;
    }

    // Funciones para reconocer diferentes tipos de tokens

    public static boolean isIdentifier(String input) {
        return Pattern.matches(IDENTIFIER_REGEX, input);
    }

    public static boolean isDigit(String input) {
        return Pattern.matches(NUMBER_REGEX, input);
    }

    public static boolean isAssignment(String input) {
        return Pattern.matches(ASSIGNMENT_REGEX, input);
    }

    public static boolean isCombinedAssignment(String input) {
        return Pattern.matches(COMBINED_ASSIGNMENT_REGEX, input);
    }

    public static boolean isOperator(String input) {
        return Pattern.matches(OPERATOR_REGEX, input);
    }

    public static boolean isLogicOperator(String input) {
        return Pattern.matches(LOGIC_OPERATOR_REGEX, input);
    }

    public static boolean isBitOperator(String input) {
        return Pattern.matches(BIT_OPERATOR_REGEX, input);
    }

    public static boolean isComparator(String input) {
        return Pattern.matches(COMPARATOR_REGEX, input);
    }

    public static boolean isDataType(String input) {
        return Pattern.matches(DATA_TYPE_REGEX, input);
    }

    public static boolean isIf(String input) {
        return Pattern.matches(IF_REGEX, input);
    }

    public static boolean isCase(String input) {
        return Pattern.matches(CASE_REGEX, input);
    }

    public static boolean isWhile(String input) {
        return Pattern.matches(WHILE_REGEX, input);
    }

    public static boolean isDo(String input) {
        return Pattern.matches(DO_REGEX, input);
    }

    public static boolean isReservedWord(String input) {
        return Pattern.matches(RESERVED_WORDS_REGEX, input);
    }

    public static boolean isString(String input) {
        if (input.equals("STR"))
            return true;
        else
            return false; // Reconoce cadenas
    }

    public static boolean isChar(String input) {
        if (input.equals("CHR"))
            return true;
        else
            return false; // Reconoce caracteres
    }

    public static boolean isLeftParen(String input) {
        return Pattern.matches(LEFT_PAREN_REGEX, input);
    }

    public static boolean isRightParen(String input) {
        return Pattern.matches(RIGHT_PAREN_REGEX, input);
    }

    public static boolean isLeftBrace(String input) {
        return Pattern.matches(LEFT_BRACE_REGEX, input);
    }

    public static boolean isRightBrace(String input) {
        return Pattern.matches(RIGHT_BRACE_REGEX, input);
    }

    public static boolean isLeftBracket(String input) {
        return Pattern.matches(LEFT_BRACKET_REGEX, input);
    }

    public static boolean isRightBracket(String input) {
        return Pattern.matches(RIGHT_BRACKET_REGEX, input);
    }

    public static boolean isComa(String input) {
        return Pattern.matches(COMA_REGEX, input);
    }

    public static boolean isDot(String input) {
        return Pattern.matches(DOT_REGEX, input);
    }

    public static boolean isSemicolon(String input) {
        return Pattern.matches(SEMICOLON_REGEX, input);
    }

    public static boolean isColon(String input) {
        return Pattern.matches(COLON_REGEX, input);
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

    public static boolean isSemiColonLine(String input) {
        return Pattern.matches(SEMICOLON_LINE_REGEX, input);
    }
    
    public static boolean isAssignLine(String input) {
        return Pattern.matches(ASSIGN_LINE_REGEX, input);
    }

    public static boolean isInitialLine(String input) {
        return Pattern.matches(INITIALIZE_LINE_REGEX, input);
    }

    public static boolean isFunctionLine(String input) {
        return Pattern.matches(FUNCTION_LINE_REGEX, input);
    }

    public static boolean isIfLine(String input) {
        return Pattern.matches(IF_LINE_REGEX, input);
    }

    public static boolean isWhileLine(String input) {
        return Pattern.matches(WHILE_LINE_REGEX, input);
    }

    public static boolean isEndLine(String input) {
        return Pattern.matches(END_LINE_REGEX, input);
    }

    public static boolean isDoLine(String input) {
        return Pattern.matches(DO_LINE_REGEX, input);
    }

    public static boolean isDoEndLine(String input) {
        return Pattern.matches(DO_END_REGEX, input);
    }
}
